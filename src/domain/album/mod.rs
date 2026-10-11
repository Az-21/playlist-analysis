use crate::domain::ids::{AlbumId, ArtistId};

/// An album, interned during parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Album {
    /// Stable id assigned during parsing
    pub id: AlbumId,
    /// Display title taken from the first track that mentioned the album
    pub title: String,
    /// Album artist, falling back to the track artist when absent
    pub artist: ArtistId,
}
