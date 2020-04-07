pub mod error;
pub mod ext;
mod map;
pub mod values;

pub use map::Map;

use crate::values::Sha1;
use derive_more::{Deref, DerefMut, From};
use std::hash::{Hash, Hasher};

pub type Result<T> = std::result::Result<T, error::Error>;

#[derive(Copy, Clone, Debug, Deref, DerefMut, From)]
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

#[derive(Clone, Debug, PartialEq, From)]
pub enum Value {
    U8(u8),
    U16(u16),
    U32(u32),
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
            Value::ShortString(_) => 3,
            Value::LongString(_) => 4,
            Value::Binary(_) => 5,
            Value::Bool(_) => 6,
            Value::Float(_) => 7,
            Value::Sha1(_) => 8,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{values::Sha1, Map, Value};
    use std::io::Cursor;

    #[test]
    fn write_and_read() {
        let mut old = Map::new();
        old.insert(0, 0u8);
        old.insert(1, 1u16);
        old.insert(2, 2u32);
        old.insert(3, Value::ShortString("short string".to_owned()));
        old.insert(4, "long string");
        old.insert(5, vec![5, 5, 5, 5, 5]);
        old.insert(6, true);
        old.insert(7, 7.7);
        old.insert(8, Sha1::new([8; 20]));

        let len = old.len();

        let mut buffer = Vec::with_capacity(len * 6);
        old.write(&mut buffer).unwrap();

        let mut new = Map::with_capacity(len);
        let mut cursor = Cursor::new(buffer);
        new.read(&mut cursor).unwrap();

        assert_eq!(old, new);
    }
}
