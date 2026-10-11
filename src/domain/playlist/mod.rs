use crate::domain::ids::{PlaylistId, SongId};

/// An ordered playlist of songs
///
/// Entries preserve playback order and may contain the same song more than once; flagging duplicates is the analysis layer's job
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Playlist {
    /// Stable id taken from the source export's playlist id
    pub id: PlaylistId,
    pub name: String,
    /// Ordered song ids
    pub entries: Vec<SongId>,
}
