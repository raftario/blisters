use crate::MAGIC_NUMBER;
use blister_format::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Format(#[from] blister_format::error::Error),
    #[error(transparent)]
    IntegerOverflow(#[from] std::num::TryFromIntError),

    #[error("invalid magic number, expected `{:?}`, got `{0:?}`", MAGIC_NUMBER)]
    InvalidMagicNumber([u8; 8]),
    #[error("invalid playlist title, expected short string, got {0:?}")]
    InvalidPlaylistTitle(Option<Value>),
    #[error("invalid playlist author, expected short string, got {0:?}")]
    InvalidPlaylistAuthor(Option<Value>),
    #[error("invalid playlist description, expected optional long string, got {0:?}")]
    InvalidPlaylistDescription(Option<Value>),
    #[error("invalid playlist cover, expected optional binary data, got {0:?}")]
    InvalidPlaylistCover(Option<Value>),

    #[error("invalid beatmap type, expected u8, got {0:?}")]
    InvalidBeatmapType(Option<Value>),
    #[error("invalid beatmap date added, expected u64, got {0:?}")]
    InvalidBeatmapDateAdded(Option<Value>),
    #[error("invalid beatmap key, expected optional u32, got {0:?}")]
    InvalidBeatmapKey(Option<Value>),
    #[error("invalid beatmap hash, expected optional SHA1 hash, got {0:?}")]
    InvalidBeatmapHash(Option<Value>),
    #[error("invalid beatmap zip, expected binary data, got {0:?}")]
    InvalidBeatmapZip(Option<Value>),
    #[error("invalid beatmap level ID, expected short string, got {0:?}")]
    InvalidBeatmapLevelId(Option<Value>),
    #[error("missing beatmap key for key identified beatmap")]
    MissingBeatmapKey,
    #[error("missing beatmap hash for hash identified beatmap")]
    MissingBeatmapHash,
    #[error("missing beatmap zip for self contained beatmap")]
    MissingBeatmapZip,
    #[error("missing beatmap level ID for level ID identified beatmap")]
    MissingBeatmapLevelId,

    #[error("encountered a beatmap with unknown type `{0}` in strict mode")]
    StrictModeUnknownBeatmapType(u8),
}
