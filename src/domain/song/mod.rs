use crate::domain::ids::{AlbumId, ArtistId, SongId};

/// A single track in the library
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Song {
    /// Stable id taken from the source export's track id
    pub id: SongId,
    pub title: String,
    /// Performing artist
    pub artist: ArtistId,
    /// Album the track belongs to
    pub album: AlbumId,
    /// Total duration in milliseconds, when the export provides it
    pub duration_ms: Option<u64>,
    /// Track number within its disc, when the export provides it
    pub track_number: Option<u32>,
    /// Disc number, when the export provides it
    pub disc_number: Option<u32>,
}
