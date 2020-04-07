use std::{io, num::TryFromIntError, string::FromUtf8Error};

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
