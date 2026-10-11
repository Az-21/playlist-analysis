//! Pure conversion from raw plist structures into the domain model

use std::collections::BTreeMap;

use crate::domain::{
    Album, AlbumId, Artist, ArtistId, Library, Playlist, PlaylistId, Song, SongId, normalize_text,
};
use crate::parser::ParseReport;

use super::raw::{RawPlaylist, RawTrack};

/// Internes raw tracks and playlists into a [`Library`], counting recoverable problems in a [`ParseReport`]
///
/// Folders are ignored
pub(super) fn convert(
    tracks: BTreeMap<String, RawTrack>,
    playlists: Vec<RawPlaylist>,
) -> (Library, ParseReport) {
    let mut library = Library::default();
    let mut report = ParseReport::default();
    let mut artists: BTreeMap<String, ArtistId> = BTreeMap::new();
    let mut albums: BTreeMap<(String, ArtistId), AlbumId> = BTreeMap::new();

    for (track_id, raw) in ordered_tracks(tracks, &mut report) {
        let (Some(title), Some(artist_name), Some(album_title)) = (raw.name, raw.artist, raw.album)
        else {
            report.skipped_tracks += 1;
            continue;
        };
        let artist = intern_artist(&mut library, &mut artists, &artist_name);
        let album_artist_name = raw.album_artist.as_deref().unwrap_or(artist_name.as_str());
        let album_artist = intern_artist(&mut library, &mut artists, album_artist_name);
        let album = intern_album(&mut library, &mut albums, &album_title, album_artist);
        let song = Song {
            id: SongId(track_id),
            title,
            artist,
            album,
            duration_ms: raw
                .total_time_ms
                .and_then(|value| u64::try_from(value).ok()),
            track_number: raw.track_number.and_then(|value| u32::try_from(value).ok()),
            disc_number: raw.disc_number.and_then(|value| u32::try_from(value).ok()),
        };
        library.songs.insert(song.id, song);
    }

    for raw in playlists {
        if raw.folder {
            continue;
        }
        let (Some(id), Some(name)) = (raw.id, raw.name) else {
            report.skipped_playlists += 1;
            continue;
        };
        let Ok(id) = u64::try_from(id) else {
            report.skipped_playlists += 1;
            continue;
        };
        if raw.items.is_empty() {
            continue;
        }
        let entries = resolve_entries(raw.items, &library, &mut report);
        library.playlists.push(Playlist {
            id: PlaylistId(id),
            name,
            entries,
        });
    }

    (library, report)
}

/// Sorts track dict entries numerically by their stringified id
fn ordered_tracks(
    tracks: BTreeMap<String, RawTrack>,
    report: &mut ParseReport,
) -> Vec<(u64, RawTrack)> {
    let mut ordered = Vec::with_capacity(tracks.len());
    for (key, track) in tracks {
        match key.parse::<u64>() {
            Ok(id) => ordered.push((id, track)),
            Err(_) => report.skipped_tracks += 1,
        }
    }
    ordered.sort_by_key(|(id, _)| *id);
    ordered
}

/// Resolves playlist item track ids against the tracks seen so far
fn resolve_entries(
    items: Vec<super::raw::RawPlaylistItem>,
    library: &Library,
    report: &mut ParseReport,
) -> Vec<SongId> {
    items
        .into_iter()
        .filter_map(|item| {
            let id = item.track_id.and_then(|value| u64::try_from(value).ok())?;
            let song = SongId(id);
            if library.songs.contains_key(&song) {
                Some(song)
            } else {
                report.skipped_playlist_entries += 1;
                None
            }
        })
        .collect()
}

/// Returns the id of the artist with the given display name, interning it on first sight
///
/// Interning keys are normalized, the display name is first-seen
fn intern_artist(
    library: &mut Library,
    index: &mut BTreeMap<String, ArtistId>,
    name: &str,
) -> ArtistId {
    let key = normalize_text(name);
    if let Some(id) = index.get(&key) {
        return *id;
    }
    let id = ArtistId(next_id(library.artists.len()));
    index.insert(key, id);
    library.artists.insert(
        id,
        Artist {
            id,
            name: name.to_owned(),
        },
    );
    id
}

/// Returns the id of the album with the given display title and album artist, interning it on first sight
///
/// Interning keys are normalized
fn intern_album(
    library: &mut Library,
    index: &mut BTreeMap<(String, ArtistId), AlbumId>,
    title: &str,
    artist: ArtistId,
) -> AlbumId {
    let key = (normalize_text(title), artist);
    if let Some(id) = index.get(&key) {
        return *id;
    }
    let id = AlbumId(next_id(library.albums.len()));
    index.insert(key, id);
    library.albums.insert(
        id,
        Album {
            id,
            title: title.to_owned(),
            artist,
        },
    );
    id
}

fn next_id(len: usize) -> u32 {
    u32::try_from(len).expect("library has more entities than u32::MAX")
}

#[cfg(test)]
mod tests;
