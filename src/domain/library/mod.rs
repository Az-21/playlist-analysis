//! The parsed library and its lookup helpers

use std::collections::BTreeMap;

use crate::domain::album::Album;
use crate::domain::artist::Artist;
use crate::domain::ids::{AlbumId, ArtistId, PlaylistId, SongId};
use crate::domain::playlist::Playlist;
use crate::domain::song::Song;

/// Everything an export provides: interned artists and albums plus ordered songs and playlists
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Library {
    /// Artists keyed by assigned id
    pub artists: BTreeMap<ArtistId, Artist>,
    /// Albums keyed by assigned id
    pub albums: BTreeMap<AlbumId, Album>,
    /// Songs keyed by source track id
    pub songs: BTreeMap<SongId, Song>,
    /// Playlists in source order
    pub playlists: Vec<Playlist>,
}

impl Library {
    #[must_use]
    pub fn song(&self, id: SongId) -> Option<&Song> {
        self.songs.get(&id)
    }

    #[must_use]
    pub fn album(&self, id: AlbumId) -> Option<&Album> {
        self.albums.get(&id)
    }

    #[must_use]
    pub fn artist(&self, id: ArtistId) -> Option<&Artist> {
        self.artists.get(&id)
    }

    #[must_use]
    pub fn playlist(&self, id: PlaylistId) -> Option<&Playlist> {
        self.playlists.iter().find(|playlist| playlist.id == id)
    }
}

/// Formats a duration in milliseconds as `m:ss`, or `h:mm:ss` beyond an hour
#[must_use]
pub fn format_duration(duration_ms: u64) -> String {
    let total_seconds = duration_ms / 1_000;
    let seconds = total_seconds % 60;
    let minutes = (total_seconds / 60) % 60;
    let hours = total_seconds / 3_600;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests;
