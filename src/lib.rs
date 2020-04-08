mod beatmap;
pub mod error;
mod playlist;

pub use crate::{
    beatmap::{Beatmap, BeatmapType},
    playlist::Playlist,
};

use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;

const MAGIC_NUMBER_LEN: usize = 8;
const MAGIC_NUMBER: &[u8; MAGIC_NUMBER_LEN] = b"Blist.v3";
