use crate::{
    ext::{ReadExt, WriteExt},
    Key, Result, Value,
};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use derive_more::{Deref, DerefMut, From};
use fnv::FnvBuildHasher;
use std::{
    collections::hash_map::{Entry, HashMap},
    convert::TryInto,
    io::{BufWriter, Read, Write},
};

#[derive(Clone, Debug, Deref, DerefMut, From)]
pub struct Map(HashMap<Key, Value, FnvBuildHasher>);

impl Map {
    pub fn read<R>(&mut self, mut reader: R) -> Result<()>
    where
        R: Read,
    {
        let len = reader.read_u32::<LE>()? as usize;
        let mut i = 0;
        while i < len {
            let (r, (k, v)) = reader.read_kv()?;
            i += r;
            self.insert(k, v);
        }
        Ok(())
    }

    pub fn write<W>(&self, mut writer: W) -> Result<()>
    where
        W: Write,
    {
        let mut buffer = BufWriter::new(Vec::with_capacity(self.len() * (4 + 1 + 1)));
        let mut i = 0;
        for (k, v) in self.iter() {
            i += buffer.write_kv(*k, v)?;
        }
        writer.write_u32::<LE>(i.try_into()?)?;
        writer.write_all(buffer.buffer())?;
        Ok(())
    }

    // HashMap overrides

    #[inline]
    pub fn new() -> Self {
        Self(HashMap::with_hasher(Default::default()))
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity_and_hasher(
            capacity,
            Default::default(),
        ))
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

impl Default for Map {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Map {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}
