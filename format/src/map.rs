use crate::{
    ext::{ReadExt, WriteExt},
    Key, Result, Value,
};
use derive_more::{Deref, DerefMut, From};
use std::{
    collections::hash_map::{Entry, HashMap, RandomState},
    hash::BuildHasher,
    io::{BufRead, Write},
};

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
