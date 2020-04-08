use crate::{error::Error, Beatmap, Result, MAGIC_NUMBER, MAGIC_NUMBER_LEN};
use blister_format::{Map, Value};
use byteorder::{ReadBytesExt, LE};
use flate2::bufread::GzDecoder;
use std::io::{BufReader, Read};

#[derive(Debug, Clone, PartialEq)]
pub struct Playlist {
    pub title: String,
    pub author: String,
    pub description: Option<String>,
    pub cover: Option<Vec<u8>>,

    pub maps: Vec<Beatmap>,

    pub custom_data: Map,
}

impl Playlist {
    pub fn read<R>(mut reader: R, strict: bool) -> Result<Self>
    where
        R: Read,
    {
        let mut magic_number = [0; MAGIC_NUMBER_LEN];
        reader.read_exact(&mut magic_number)?;
        if !constant_time_eq::constant_time_eq(&magic_number[..], &MAGIC_NUMBER[..]) {
            return Err(Error::InvalidMagicNumber(magic_number));
        }

        let mut decoder = GzDecoder::new(BufReader::new(reader));

        let mut data = Map::with_capacity(2);
        data.read(&mut decoder)?;

        let title = match data.remove(0) {
            Some(Value::ShortString(s)) => s,
            v => return Err(Error::InvalidPlaylistTitle(v)),
        };
        let author = match data.remove(1) {
            Some(Value::ShortString(s)) => s,
            v => return Err(Error::InvalidPlaylistAuthor(v)),
        };
        let description = match data.remove(2) {
            Some(Value::LongString(s)) => Some(s),
            None => None,
            v => return Err(Error::InvalidPlaylistDescription(v)),
        };
        let cover = match data.remove(3) {
            Some(Value::Binary(b)) => Some(b),
            None => None,
            v => return Err(Error::InvalidPlaylistCover(v)),
        };

        let map_count = decoder.read_u32::<LE>()? as usize;
        let mut maps = Vec::with_capacity(map_count);
        for _ in 0..map_count {
            maps.push(Beatmap::read(&mut decoder, strict)?);
        }

        Ok(Self {
            title,
            author,
            description,
            cover,
            maps,
            custom_data: data,
        })
    }
}
