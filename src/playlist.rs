use crate::{error::Error, Beatmap, Result, MAGIC_NUMBER, MAGIC_NUMBER_LEN};
use blister_format::{Map, Value};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use flate2::{bufread::GzDecoder, write::GzEncoder, Compression};
use std::{
    convert::TryInto,
    io::{BufReader, Read, Write},
};

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
    pub fn new(title: String, author: String) -> Self {
        Self {
            title,
            author,
            description: None,
            cover: None,
            maps: Default::default(),
            custom_data: Default::default(),
        }
    }

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

    #[inline]
    pub fn write<W>(self, writer: W) -> Result<()>
    where
        W: Write,
    {
        self.write_with_compression(writer, Default::default())
    }

    pub fn write_with_compression<W>(self, mut writer: W, level: Compression) -> Result<()>
    where
        W: Write,
    {
        writer.write_all(MAGIC_NUMBER)?;

        let mut encoder = GzEncoder::new(
            Vec::with_capacity((4 + 1 + 1 + 1) + (4 + 1 + 1 + 1) + 4),
            level,
        );

        let Self {
            title,
            author,
            description,
            cover,
            maps,
            custom_data: mut data,
        } = self;

        data.insert(0, Value::ShortString(title));
        data.insert(1, Value::ShortString(author));
        if let Some(s) = description {
            data.insert(2, Value::LongString(s));
        }
        if let Some(b) = cover {
            data.insert(3, Value::Binary(b));
        }
        data.write(&mut encoder)?;

        let map_count = maps.len();
        encoder.write_u32::<LE>(map_count.try_into()?)?;
        for map in maps {
            map.write(&mut encoder)?;
        }

        writer.write_all(&encoder.finish()?)?;
        Ok(())
    }
}
