use crate::{error::Error, Result};
use blister_format::{values::Sha1, Map, Value};
use chrono::{DateTime, TimeZone, Utc};
use std::convert::TryInto;
use std::io::Read;

#[derive(Debug, Clone, PartialEq)]
pub struct Beatmap {
    pub ty: BeatmapType,
    pub date_added: DateTime<Utc>,

    pub key: Option<u32>,
    pub hash: Option<Sha1>,
    pub zip: Option<Vec<u8>>,
    pub level_id: Option<String>,

    pub custom_data: Map,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum BeatmapType {
    Key = 0,
    Hash = 1,
    Zip = 2,
    LevelId = 3,

    Unknown = 255,
}

impl From<u8> for BeatmapType {
    fn from(u: u8) -> Self {
        match u {
            0 => Self::Key,
            1 => Self::Hash,
            2 => Self::Zip,
            3 => Self::LevelId,
            _ => Self::Unknown,
        }
    }
}

impl Beatmap {
    pub(crate) fn read<R>(mut reader: R, strict: bool) -> Result<Self>
    where
        R: Read,
    {
        let mut data = Map::with_capacity(2);
        data.read(&mut reader)?;

        let ty = match data.remove(0) {
            Some(Value::U8(u)) => {
                let ty = u.into();
                if strict && ty == BeatmapType::Unknown {
                    return Err(Error::StrictModeUnknownBeatmapType(u));
                }
                ty
            }
            v => return Err(Error::InvalidBeatmapType(v)),
        };
        let date_added = match data.remove(1) {
            Some(Value::U64(u)) => Utc.timestamp(u.try_into()?, 0),
            v => return Err(Error::InvalidBeatmapDateAdded(v)),
        };

        let key = match data.remove(2) {
            Some(Value::U32(u)) => Some(u),
            None => None,
            v => return Err(Error::InvalidBeatmapKey(v)),
        };
        let hash = match data.remove(3) {
            Some(Value::Sha1(h)) => Some(h),
            None => None,
            v => return Err(Error::InvalidBeatmapHash(v)),
        };
        let zip = match data.remove(4) {
            Some(Value::Binary(b)) => Some(b),
            None => None,
            v => return Err(Error::InvalidBeatmapZip(v)),
        };
        let level_id = match data.remove(5) {
            Some(Value::ShortString(s)) => Some(s),
            None => None,
            v => return Err(Error::InvalidBeatmapLevelId(v)),
        };

        match ty {
            BeatmapType::Key => {
                if key.is_none() {
                    return Err(Error::MissingBeatmapKey);
                }
            }
            BeatmapType::Hash => {
                if hash.is_none() {
                    return Err(Error::MissingBeatmapHash);
                }
            }
            BeatmapType::Zip => {
                if zip.is_none() {
                    return Err(Error::MissingBeatmapZip);
                }
            }
            BeatmapType::LevelId => {
                if level_id.is_none() {
                    return Err(Error::MissingBeatmapLevelId);
                }
            }
            _ => (),
        }

        Ok(Self {
            ty,
            date_added,

            key,
            hash,
            zip,
            level_id,

            custom_data: data,
        })
    }
}
