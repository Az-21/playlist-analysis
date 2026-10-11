//! Raw serde structures mirroring the plist layout of an Apple Music export

use std::collections::BTreeMap;

use serde::Deserialize;

/// The top-level plist dict, reduced to the sections the tool needs
#[derive(Debug, Deserialize)]
pub(super) struct RawLibrary {
    /// Track dictionaries keyed by the stringified track id
    #[serde(rename = "Tracks")]
    pub(super) tracks: Option<BTreeMap<String, RawTrack>>,
    /// Playlists in export order
    #[serde(rename = "Playlists")]
    pub(super) playlists: Option<Vec<RawPlaylist>>,
}

/// A single entry of the `Tracks` dict; keys the tool does not need are ignored
#[derive(Debug, Deserialize)]
pub(super) struct RawTrack {
    #[serde(rename = "Name")]
    pub(super) name: Option<String>,
    #[serde(rename = "Artist")]
    pub(super) artist: Option<String>,
    #[serde(rename = "Album Artist")]
    pub(super) album_artist: Option<String>,
    #[serde(rename = "Album")]
    pub(super) album: Option<String>,
    #[serde(rename = "Total Time")]
    pub(super) total_time_ms: Option<i64>,
    #[serde(rename = "Track Number")]
    pub(super) track_number: Option<i64>,
    #[serde(rename = "Disc Number")]
    pub(super) disc_number: Option<i64>,
}

/// A single entry of the `Playlists` array
#[derive(Debug, Deserialize)]
pub(super) struct RawPlaylist {
    #[serde(rename = "Playlist ID")]
    pub(super) id: Option<i64>,
    #[serde(rename = "Name")]
    pub(super) name: Option<String>,
    #[serde(rename = "Folder", default)]
    pub(super) folder: bool,
    #[serde(rename = "Playlist Items", default)]
    pub(super) items: Vec<RawPlaylistItem>,
}

/// A single playlist item referencing a track
#[derive(Debug, Deserialize)]
pub(super) struct RawPlaylistItem {
    #[serde(rename = "Track ID")]
    pub(super) track_id: Option<i64>,
}
