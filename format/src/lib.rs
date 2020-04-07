use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use derive_more::{Deref, DerefMut, From};
use std::collections::HashMap;
use std::{
    collections::hash_map::{Entry, RandomState},
    convert::TryInto,
    hash::{BuildHasher, Hash, Hasher},
    io::{self, BufRead, Read, Write},
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

#[derive(Clone, Debug, Deref, DerefMut, From)]
pub struct Map<S>(HashMap<Key, Value, S>)
where
    S: BuildHasher;

impl Map<RandomState> {
    #[inline]
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }
}

impl<S> Map<S>
where
    S: BuildHasher,
{
    pub fn read<R>(&mut self, mut reader: R) -> Result<()>
    where
        R: BufRead,
    {
        let mut buffer = reader.fill_buf()?;
        let mut len = buffer.len();
        while len > 0 {
            let (k, v) = reader.read_kv()?;
            self.insert(k, v);

            buffer = reader.fill_buf()?;
            len = buffer.len();
        }

        Ok(())
    }

    pub fn write<W>(&self, mut writer: W) -> Result<()>
    where
        W: Write,
    {
        for (k, v) in self.iter() {
            writer.write_kv(*k, v)?;
        }
        Ok(())
    }

    // HashMap overrides

    #[inline]
    pub fn with_hasher(hash_builder: S) -> Self {
        Self(HashMap::with_hasher(hash_builder))
    }

    #[inline]
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        Self(HashMap::with_capacity_and_hasher(capacity, hash_builder))
    }

    #[inline]
    pub fn entry<K>(&mut self, key: K) -> Entry<Key, Value>
    where
        K: Into<Key>,
    {
        self.0.entry(key.into())
    }

    #[inline]
    pub fn get<K>(&self, key: K) -> Option<&Value>
    where
        K: Into<Key>,
    {
        self.0.get(&key.into())
    }

    #[inline]
    pub fn get_key_value<K>(&self, key: K) -> Option<(Key, &Value)>
    where
        K: Into<Key>,
    {
        self.0.get_key_value(&key.into()).map(|(k, v)| (*k, v))
    }

    #[inline]
    pub fn contains_key<K>(&self, key: K) -> bool
    where
        K: Into<Key>,
    {
        self.0.contains_key(&key.into())
    }

    #[inline]
    pub fn get_mut<K>(&mut self, key: K) -> Option<&mut Value>
    where
        K: Into<Key>,
    {
        self.0.get_mut(&key.into())
    }

    #[inline]
    pub fn insert<K, V>(&mut self, key: K, value: V) -> Option<Value>
    where
        K: Into<Key>,
        V: Into<Value>,
    {
        self.0.insert(key.into(), value.into())
    }

    #[inline]
    pub fn remove<K>(&mut self, key: K) -> Option<Value>
    where
        K: Into<Key>,
    {
        self.0.remove(&key.into())
    }

    #[inline]
    pub fn remove_entry<K>(&mut self, key: K) -> Option<(Key, Value)>
    where
        K: Into<Key>,
    {
        self.0.remove_entry(&key.into())
    }
}

impl Default for Map<RandomState> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<S> PartialEq for Map<S>
where
    S: BuildHasher,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

#[cfg(test)]
mod tests {
    use crate::{Map, Sha1, Value};
    use std::io::Cursor;

    #[test]
    fn write_and_read() {
        let mut old = Map::new();
        old.insert(0, 0u8);
        old.insert(1, 0u16);
        old.insert(2, 0u32);
        old.insert(3, Value::ShortString("short string".to_owned()));
        old.insert(4, "long string");
        old.insert(5, vec![0, 1, 2, 3, 4, 5]);
        old.insert(6, true);
        old.insert(7, 7.7);
        old.insert(8, Sha1([0; 20]));

        let len = old.len();

        let mut buffer = Vec::with_capacity(len * 6);
        old.write(&mut buffer).unwrap();

        let mut new = Map::with_capacity(len);
        let mut cursor = Cursor::new(buffer);
        new.read(&mut cursor).unwrap();

        assert_eq!(old, new);
    }
}
