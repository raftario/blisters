use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    IntegerOverflow(#[from] std::num::TryFromIntError),

    #[error("`{0} isn't a valid data type`")]
    InvalidDataType(u8),
    #[error("`{0} isn't a valid boolean, should be `0` for false or `1` for true`")]
    InvalidBoolean(u8),
}
