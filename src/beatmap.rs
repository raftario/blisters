use crate::{error::Error, Result};
use blister_format::{values::Sha1, Map, Value};
use chrono::{DateTime, TimeZone, Utc};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::{
    convert::TryInto,
    io::{Read, Write},
};

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
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, IntoPrimitive, TryFromPrimitive)]
pub enum BeatmapType {
    Key = 0,
    Hash = 1,
    Zip = 2,
    LevelId = 3,

    Unknown = 255,
}

impl BeatmapType {
    #[inline]
    fn from(u: u8) -> Self {
        match u {
            0..=3 => u.try_into().unwrap(),
            _ => Self::Unknown,
        }
    }
}

impl Beatmap {
    pub fn new_key(key: u32) -> Self {
        Self {
            ty: BeatmapType::Key,
            date_added: Utc::now(),
            key: Some(key),
            hash: None,
            zip: None,
            level_id: None,
            custom_data: Default::default(),
        }
    }

    pub fn new_hash(hash: Sha1) -> Self {
        Self {
            ty: BeatmapType::Hash,
            date_added: Utc::now(),
            key: None,
            hash: Some(hash),
            zip: None,
            level_id: None,
            custom_data: Default::default(),
        }
    }

    pub fn new_zip(zip: Vec<u8>) -> Self {
        Self {
            ty: BeatmapType::Zip,
            date_added: Utc::now(),
            key: None,
            hash: None,
            zip: Some(zip),
            level_id: None,
            custom_data: Default::default(),
        }
    }

    pub fn new_level_id(level_id: String) -> Self {
        Self {
            ty: BeatmapType::LevelId,
            date_added: Utc::now(),
            key: None,
            hash: None,
            zip: None,
            level_id: Some(level_id),
            custom_data: Default::default(),
        }
    }

    pub(crate) fn read<R>(mut reader: R, strict: bool) -> Result<Self>
    where
        R: Read,
    {
        let mut data = Map::with_capacity(2);
        data.read(&mut reader)?;

        let ty = match data.remove(0) {
            Some(Value::U8(u)) => {
                let ty = BeatmapType::from(u);
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

    pub(crate) fn write<W>(self, mut writer: W) -> Result<()>
    where
        W: Write,
    {
        let Self {
            ty,
            date_added,
            key,
            hash,
            zip,
            level_id,
            custom_data: mut data,
        } = self;

        data.insert(0, Value::U8(ty.into()));
        data.insert(1, Value::U64(date_added.timestamp().try_into()?));
        if let Some(u) = key {
            data.insert(2, Value::U32(u));
        }
        if let Some(h) = hash {
            data.insert(3, Value::Sha1(h));
        }
        if let Some(b) = zip {
            data.insert(4, Value::Binary(b));
        }
        if let Some(s) = level_id {
            data.insert(5, Value::ShortString(s));
        }

        data.write(&mut writer)?;
        Ok(())
    }
}
