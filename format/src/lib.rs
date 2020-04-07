use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use derive_more::{Deref, DerefMut, From};
use std::{
    convert::TryInto,
    io::{self, Read, Write},
    num::TryFromIntError,
    string::FromUtf8Error,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error("`{0} isn't a valid data type`")]
    InvalidDataType(u8),
    #[error("`{0} isn't a valid boolean, should be `0` for false or `1` for true`")]
    InvalidBoolean(u8),
    #[error(transparent)]
    InvalidUtf8(#[from] FromUtf8Error),
    #[error(transparent)]
    IntegerOverflow(#[from] TryFromIntError),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Deref, DerefMut, From)]
pub struct Key(u32);

#[derive(Copy, Clone, Debug, Deref, DerefMut, From)]
pub struct Sha1([u8; 20]);

impl PartialEq for Sha1 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        constant_time_eq::constant_time_eq(&self[..], &other[..])
    }
}
impl Eq for Sha1 {}

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

pub trait ReadExt: Read {
    fn read_kv(&mut self) -> Result<(Key, Value)> {
        let key = Key(self.read_u32::<LE>()?);

        let data_type = self.read_u8()?;
        let value = match data_type {
            0 => Value::U8(self.read_u8()?),
            1 => Value::U16(self.read_u16::<LE>()?),
            2 => Value::U32(self.read_u32::<LE>()?),
            3 => {
                let len = self.read_u8()? as usize;
                let mut utf8 = vec![0; len];
                self.read_exact(&mut utf8)?;
                Value::ShortString(String::from_utf8(utf8)?)
            }
            4 => {
                let len = self.read_u16::<LE>()? as usize;
                let mut utf8 = vec![0; len];
                self.read_exact(&mut utf8)?;
                Value::LongString(String::from_utf8(utf8)?)
            }
            5 => {
                let len = self.read_u32::<LE>()? as usize;
                let mut bytes = vec![0; len];
                self.read_exact(&mut bytes)?;
                Value::Binary(bytes)
            }
            6 => {
                let value = self.read_u8()?;
                match value {
                    0 => Value::Bool(false),
                    1 => Value::Bool(true),
                    _ => return Err(Error::InvalidBoolean(value)),
                }
            }
            7 => Value::Float(self.read_f32::<LE>()?),
            8 => {
                let mut sha1 = [0; 20];
                self.read_exact(&mut sha1)?;
                Value::Sha1(Sha1(sha1))
            }
            _ => return Err(Error::InvalidDataType(data_type)),
        };

        Ok((key, value))
    }
}

impl<R> ReadExt for R where R: Read + ?Sized {}

pub trait WriteExt: Write {
    fn write_kv<'a, K, V>(&mut self, key: K, value: V) -> Result<()>
    where
        K: Into<Key>,
        V: Into<&'a Value>,
    {
        let key = key.into();
        let value = value.into();

        self.write_u32::<LE>(*key)?;

        self.write_u8(value.data_type())?;
        match value {
            Value::U8(v) => self.write_u8(*v)?,
            Value::U16(v) => self.write_u16::<LE>(*v)?,
            Value::U32(v) => self.write_u32::<LE>(*v)?,
            Value::ShortString(v) => {
                let utf8 = v.clone().into_bytes();
                let len = utf8.len().try_into()?;
                self.write_u8(len)?;
                self.write_all(&utf8)?;
            }
            Value::LongString(v) => {
                let utf8 = v.clone().into_bytes();
                let len = utf8.len().try_into()?;
                self.write_u16::<LE>(len)?;
                self.write_all(&utf8)?;
            }
            Value::Binary(v) => {
                let len = v.len().try_into()?;
                self.write_u32::<LE>(len)?;
                self.write_all(&v)?;
            }
            Value::Bool(v) => {
                let value = if *v { 1 } else { 0 };
                self.write_u8(value)?;
            }
            Value::Float(v) => self.write_f32::<LE>(*v)?,
            Value::Sha1(v) => self.write_all(&v[..])?,
        }

        Ok(())
    }
}

impl<W> WriteExt for W where W: Write + ?Sized {}

#[cfg(test)]
mod tests {
    use crate::{Key, ReadExt, Sha1, Value, WriteExt};
    use std::{collections::HashMap, io::Cursor};

    #[test]
    fn write_and_read() {
        let mut old: HashMap<Key, Value> = HashMap::new();
        old.insert(0.into(), 0u8.into());
        old.insert(1.into(), 0u16.into());
        old.insert(2.into(), 0u32.into());
        old.insert(3.into(), Value::ShortString("short string".to_owned()));
        old.insert(4.into(), "long string".into());
        old.insert(5.into(), vec![0, 1, 2, 3, 4, 5].into());
        old.insert(6.into(), true.into());
        old.insert(7.into(), 7.7.into());
        old.insert(8.into(), Sha1([0; 20]).into());

        let len = old.len();

        let mut buffer = Vec::with_capacity(len * 6);
        for (k, v) in old.iter() {
            buffer.write_kv(*k, v).unwrap();
        }

        let mut new: HashMap<Key, Value> = HashMap::with_capacity(len);
        let mut cursor = Cursor::new(buffer);
        for _ in 0..len {
            let (k, v) = cursor.read_kv().unwrap();
            new.insert(k, v);
        }

        assert_eq!(old, new);
    }
}
