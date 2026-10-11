//! Duplicate songs within a playlist

use std::collections::BTreeMap;

use crate::analysis::Analysis;
use crate::analysis::report::{DuplicateEntries, DuplicateOccurrence, Finding};
use crate::domain::{AlbumId, ArtistId, Library, Playlist, normalize_text};

/// Flags songs that appear more than once in a playlist
///
/// Entries are grouped by normalized title, artist, and album, so both a repeated track id and distinct track ids with identical metadata count
#[derive(Debug, Default, Clone, Copy)]
pub struct DuplicateSongs;

impl Analysis for DuplicateSongs {
    fn id(&self) -> &'static str {
        "duplicate-songs"
    }

    fn title(&self) -> &'static str {
        "Duplicate Songs"
    }

    fn analyze(&self, library: &Library) -> Vec<Finding> {
        library
            .playlists
            .iter()
            .flat_map(|playlist| duplicates_in(library, playlist))
            .collect()
    }
}

fn duplicates_in(library: &Library, playlist: &Playlist) -> Vec<Finding> {
    let mut groups: BTreeMap<(String, ArtistId, AlbumId), Vec<DuplicateOccurrence>> =
        BTreeMap::new();
    for (position, song_id) in playlist.entries.iter().enumerate() {
        let Some(song) = library.song(*song_id) else {
            continue;
        };
        let key = (normalize_text(&song.title), song.artist, song.album);
        groups.entry(key).or_default().push(DuplicateOccurrence {
            position,
            song: *song_id,
        });
    }

    let mut duplicates: Vec<DuplicateEntries> = groups
        .into_values()
        .filter(|occurrences| occurrences.len() > 1)
        .filter_map(|occurrences| {
            let first = occurrences.first()?;
            let song = library.song(first.song)?;
            Some(DuplicateEntries {
                playlist: playlist.id,
                title: song.title.clone(),
                artist: song.artist,
                album: song.album,
                occurrences,
            })
        })
        .collect();
    duplicates.sort_by_key(|duplicate| {
        duplicate
            .occurrences
            .first()
            .map_or(usize::MAX, |occurrence| occurrence.position)
    });
    duplicates
        .into_iter()
        .map(Finding::DuplicateEntries)
        .collect()
}

#[cfg(test)]
mod tests;
