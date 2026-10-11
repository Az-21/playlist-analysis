//! Detection of albums that look like different versions of one another

use std::collections::{BTreeMap, BTreeSet};

use strsim::normalized_damerau_levenshtein;

use crate::analysis::Analysis;
use crate::analysis::report::Finding;
use crate::analysis::report::SimilarAlbums as SimilarAlbumsFinding;
use crate::domain::{Album, AlbumId, Library, base_text, normalize_text};

use super::union_find::DisjointSet;

/// Minimum similarity for two full album titles to count as versions without decoration tags
const TITLE_SIMILARITY_THRESHOLD: f64 = 0.8;

/// Flags albums whose titles match one another
///
/// The analysis covers the whole library: every album referenced by any playlist is compared by title alone, so mixed use of standard, deluxe, or remastered versions surfaces even when the tracks or artists differ
#[derive(Debug, Default, Clone, Copy)]
pub struct SimilarAlbums;

impl Analysis for SimilarAlbums {
    fn id(&self) -> &'static str {
        "similar-albums"
    }

    fn title(&self) -> &'static str {
        "Similar Albums"
    }

    fn analyze(&self, library: &Library) -> Vec<Finding> {
        let candidates = candidate_albums(library);
        let keys: Vec<MatchKey> = candidates
            .iter()
            .map(|album| MatchKey::new(album))
            .collect();
        let mut groups = DisjointSet::new(candidates.len());
        for (left, key) in keys.iter().enumerate() {
            for (right, other) in keys.iter().enumerate().skip(left + 1) {
                if key.matches(other) {
                    groups.union(left, right);
                }
            }
        }
        findings(&candidates, &mut groups)
    }
}

/// Albums with at least one song referenced by a playlist, ordered by id
fn candidate_albums(library: &Library) -> Vec<&Album> {
    let album_ids: BTreeSet<AlbumId> = library
        .playlists
        .iter()
        .flat_map(|playlist| &playlist.entries)
        .filter_map(|song_id| library.song(*song_id))
        .map(|song| song.album)
        .collect();
    album_ids
        .into_iter()
        .filter_map(|album_id| library.album(album_id))
        .collect()
}

/// Comparison key of one album: normalized title and decoration-stripped title
struct MatchKey {
    title: String,
    base_title: String,
}

impl MatchKey {
    fn new(album: &Album) -> Self {
        Self {
            title: normalize_text(&album.title),
            base_title: base_text(&album.title),
        }
    }

    fn matches(&self, other: &Self) -> bool {
        (!self.base_title.is_empty() && self.base_title == other.base_title)
            || similar(&self.title, &other.title, TITLE_SIMILARITY_THRESHOLD)
    }
}

fn similar(left: &str, right: &str, threshold: f64) -> bool {
    normalized_damerau_levenshtein(left, right) >= threshold
}

/// Turns the union-find groups into one finding per cluster of two or more albums
fn findings(candidates: &[&Album], groups: &mut DisjointSet) -> Vec<Finding> {
    let mut clusters: BTreeMap<usize, Vec<&Album>> = BTreeMap::new();
    for (index, album) in candidates.iter().copied().enumerate() {
        clusters.entry(groups.find(index)).or_default().push(album);
    }
    clusters
        .into_values()
        .filter(|albums| albums.len() > 1)
        .map(|albums| {
            Finding::SimilarAlbums(SimilarAlbumsFinding {
                albums: albums.into_iter().map(|album| album.id).collect(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
