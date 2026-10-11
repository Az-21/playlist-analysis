//! Songs sharing a title and artist across different albums

use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::Analysis;
use crate::analysis::report::Finding;
use crate::analysis::report::SameTitleDifferentAlbum as SameTitleDifferentAlbumFinding;
use crate::domain::{AlbumId, ArtistId, Library, SongId, normalize_text};

/// Flags songs that share a title and artist but appear on different albums
///
/// The candidate set is every song referenced by at least one playlist, deduplicated by song id and compared across the whole library
#[derive(Debug, Default, Clone, Copy)]
pub struct SameTitleDifferentAlbum;

impl Analysis for SameTitleDifferentAlbum {
    fn id(&self) -> &'static str {
        "same-title-different-album"
    }

    fn title(&self) -> &'static str {
        "Same Title, Different Album"
    }

    fn analyze(&self, library: &Library) -> Vec<Finding> {
        let mut groups: BTreeMap<(String, ArtistId), SongGroup> = BTreeMap::new();
        let mut visited: BTreeSet<SongId> = BTreeSet::new();
        for playlist in &library.playlists {
            for song_id in &playlist.entries {
                if !visited.insert(*song_id) {
                    continue;
                }
                let Some(song) = library.song(*song_id) else {
                    continue;
                };
                let group = groups
                    .entry((normalize_text(&song.title), song.artist))
                    .or_insert_with(|| SongGroup {
                        display_title: song.title.clone(),
                        albums: BTreeSet::new(),
                        songs: Vec::new(),
                    });
                group.albums.insert(song.album);
                group.songs.push(song.id);
            }
        }
        groups
            .into_iter()
            .filter(|(_, group)| group.albums.len() > 1)
            .map(|((_, artist), group)| {
                Finding::SameTitleDifferentAlbum(SameTitleDifferentAlbumFinding {
                    title: group.display_title,
                    artist,
                    albums: group.albums.into_iter().collect(),
                    songs: group.songs,
                })
            })
            .collect()
    }
}

struct SongGroup {
    display_title: String,
    albums: BTreeSet<AlbumId>,
    songs: Vec<SongId>,
}

#[cfg(test)]
mod tests;
