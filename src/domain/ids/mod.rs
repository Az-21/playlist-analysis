//! Newtype identifiers for domain entities

use std::fmt;

/// Identifier of a song, taken from the source export's track id
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SongId(pub u64);

/// Identifier of an album, assigned in first-seen order during parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AlbumId(pub u32);

/// Identifier of an artist, assigned in first-seen order during parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArtistId(pub u32);

/// Identifier of a playlist, taken from the source export's playlist id
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlaylistId(pub u64);

macro_rules! impl_display {
    ($($id:ty),+) => {
        $(
            impl fmt::Display for $id {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    self.0.fmt(f)
                }
            }
        )+
    };
}

impl_display!(SongId, AlbumId, ArtistId, PlaylistId);
