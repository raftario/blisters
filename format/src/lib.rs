pub mod error;
pub mod ext;
mod map;
pub mod values;

pub use map::Map;

use crate::{error::Error, values::Sha1};
use derive_more::{Deref, DerefMut, From};
use std::hash::{Hash, Hasher};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Copy, Clone, Deref, DerefMut, From)]
pub struct Key(u32);

impl PartialEq for Key {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}
impl Eq for Key {}

impl Hash for Key {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state)
    }
}

#[derive(Debug, Clone, PartialEq, From)]
pub enum Value {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    #[from(ignore)]
    ShortString(String),
    LongString(String),
    Binary(Vec<u8>),
    Bool(bool),
    Float(f32),
    Sha1(Sha1),
}

impl std::convert::From<&str> for Value {
    fn from(s: &str) -> Self {
        Self::LongString(s.to_owned())
    }
}

impl Value {
    fn data_type(&self) -> u8 {
        match self {
            Value::U8(_) => 0,
            Value::U16(_) => 1,
            Value::U32(_) => 2,
            Value::U64(_) => 3,
            Value::ShortString(_) => 4,
            Value::LongString(_) => 5,
            Value::Binary(_) => 6,
            Value::Bool(_) => 7,
            Value::Float(_) => 8,
            Value::Sha1(_) => 9,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{values::Sha1, Map, Value};

    #[test]
    fn write_and_read() {
        let mut old = Map::new();
        old.insert(0, 0u8);
        old.insert(1, 1u16);
        old.insert(2, 2u32);
        old.insert(3, 3u64);
        old.insert(4, Value::ShortString("short string".to_owned()));
        old.insert(5, "long string");
        old.insert(6, vec![6, 6, 6, 6, 6, 6]);
        old.insert(7, true);
        old.insert(8, 8.8);
        old.insert(9, Sha1([9; 20]));

        let len = old.len();

        let mut buffer = Vec::new();
        old.write(&mut buffer).unwrap();

        let mut new = Map::with_capacity(len);
        new.read(buffer.as_slice()).unwrap();

        assert_eq!(old, new);
    }
}
