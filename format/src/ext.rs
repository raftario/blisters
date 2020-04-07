use crate::{error::Error, values::Sha1, Key, Result, Value};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::{
    convert::TryInto,
    io::{Read, Write},
};

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
                let mut hash = [0; 20];
                self.read_exact(&mut hash)?;
                Value::Sha1(Sha1::new(hash))
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
