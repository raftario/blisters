use crate::{error::Error, values::Sha1, Key, Result, Value};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::{
    convert::TryInto,
    io::{Read, Write},
};

pub trait ReadExt: Read {
    fn read_kv(&mut self) -> Result<(usize, (Key, Value))> {
        let read;

        let key = Key(self.read_u32::<LE>()?);

        let data_type = self.read_u8()?;
        let value = match data_type {
            0 => {
                read = 4 + 1 + 1;
                Value::U8(self.read_u8()?)
            }
            1 => {
                read = 4 + 1 + 2;
                Value::U16(self.read_u16::<LE>()?)
            }
            2 => {
                read = 4 + 1 + 4;
                Value::U32(self.read_u32::<LE>()?)
            }
            3 => {
                let len = self.read_u8()? as usize;
                let mut utf8 = vec![0; len];
                self.read_exact(&mut utf8)?;

                read = 4 + 1 + 1 + len;
                Value::ShortString(String::from_utf8(utf8)?)
            }
            4 => {
                let len = self.read_u16::<LE>()? as usize;
                let mut utf8 = vec![0; len];
                self.read_exact(&mut utf8)?;

                read = 4 + 1 + 2 + len;
                Value::LongString(String::from_utf8(utf8)?)
            }
            5 => {
                let len = self.read_u32::<LE>()? as usize;
                let mut bytes = vec![0; len];
                self.read_exact(&mut bytes)?;

                read = 4 + 1 + 4 + len;
                Value::Binary(bytes)
            }
            6 => {
                let value = self.read_u8()?;

                read = 4 + 1 + 1;
                match value {
                    0 => Value::Bool(false),
                    1 => Value::Bool(true),
                    _ => return Err(Error::InvalidBoolean(value)),
                }
            }
            7 => {
                read = 4 + 1 + 4;
                Value::Float(self.read_f32::<LE>()?)
            }
            8 => {
                let mut hash = [0; 20];
                self.read_exact(&mut hash)?;

                read = 4 + 1 + 20;
                Value::Sha1(Sha1::new(hash))
            }
            _ => return Err(Error::InvalidDataType(data_type)),
        };

        Ok((read, (key, value)))
    }
}
impl<R> ReadExt for R where R: Read + ?Sized {}

pub trait WriteExt: Write {
    fn write_kv(&mut self, key: Key, value: &Value) -> Result<usize> {
        let written;

        self.write_u32::<LE>(*key)?;

        self.write_u8(value.data_type())?;
        match value {
            Value::U8(v) => {
                self.write_u8(*v)?;
                written = 4 + 1 + 1;
            }
            Value::U16(v) => {
                self.write_u16::<LE>(*v)?;
                written = 4 + 1 + 2;
            }
            Value::U32(v) => {
                self.write_u32::<LE>(*v)?;
                written = 4 + 1 + 4;
            }
            Value::ShortString(v) => {
                let utf8 = v.clone().into_bytes();
                let len = utf8.len();
                self.write_u8(len.try_into()?)?;
                self.write_all(&utf8)?;

                written = 4 + 1 + 1 + len;
            }
            Value::LongString(v) => {
                let utf8 = v.clone().into_bytes();
                let len = utf8.len();
                self.write_u16::<LE>(len.try_into()?)?;
                self.write_all(&utf8)?;

                written = 4 + 1 + 2 + len;
            }
            Value::Binary(v) => {
                let len = v.len();
                self.write_u32::<LE>(len.try_into()?)?;
                self.write_all(&v)?;

                written = 4 + 1 + 4 + len;
            }
            Value::Bool(v) => {
                let value = if *v { 1 } else { 0 };
                self.write_u8(value)?;

                written = 4 + 1 + 1;
            }
            Value::Float(v) => {
                self.write_f32::<LE>(*v)?;
                written = 4 + 1 + 4;
            }
            Value::Sha1(v) => {
                self.write_all(&v[..])?;
                written = 4 + 1 + 20;
            }
        }

        Ok(written)
    }
}
impl<W> WriteExt for W where W: Write + ?Sized {}
