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

#[cfg(test)]
mod tests {
    use crate::{Beatmap, Playlist};
    use chrono::{TimeZone, Utc};

    #[test]
    fn write_and_read() {
        let mut old = Playlist::new("test playlist".to_owned(), "me".to_owned());
        old.description = Some("description".to_owned());
        old.cover = Some(vec![2, 1, 1, 2]);
        old.custom_data.insert(2112, 1.234);

        old.maps.push(Beatmap::new_key(2112));
        old.maps.push(Beatmap::new_hash([4; 20].into()));
        old.maps
            .push(Beatmap::new_zip(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]));
        old.maps.push(Beatmap::new_level_id("level ID".to_owned()));

        for m in old.maps.iter_mut() {
            m.date_added = Utc.timestamp(m.date_added.timestamp(), 0);
        }

        let mut buffer = Vec::new();
        old.clone().write(&mut buffer).unwrap();

        let new = Playlist::read(buffer.as_slice(), true).unwrap();

        assert_eq!(old, new);
    }
}
