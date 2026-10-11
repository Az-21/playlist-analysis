//! Fuzzy detection of alternative versions of the same song

use std::collections::{BTreeMap, BTreeSet};

use strsim::normalized_damerau_levenshtein;

use crate::analysis::Analysis;
use crate::analysis::report::AlternativeVersions as AlternativeVersionsFinding;
use crate::analysis::report::Finding;
use crate::domain::{Library, Song, base_text};

use super::union_find::DisjointSet;

/// Minimum similarity for two songs to count as possible alternative versions
const SIMILARITY_THRESHOLD: f64 = 0.5;

/// Flags songs whose title, artist, and album closely resemble one another
#[derive(Debug, Default, Clone, Copy)]
pub struct AlternativeVersions;

impl Analysis for AlternativeVersions {
    fn id(&self) -> &'static str {
        "alternative-versions"
    }

    fn title(&self) -> &'static str {
        "Alternative Versions"
    }

    fn analyze(&self, library: &Library) -> Vec<Finding> {
        let candidates = candidate_songs(library);
        let keys: Vec<MatchKey> = candidates
            .iter()
            .map(|song| MatchKey::new(song, library))
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

/// Songs referenced by at least one playlist, deduplicated by id and ordered by id
fn candidate_songs(library: &Library) -> Vec<&Song> {
    let mut seen = BTreeSet::new();
    let mut songs = Vec::new();
    for playlist in &library.playlists {
        for song_id in &playlist.entries {
            if !seen.insert(*song_id) {
                continue;
            }
            if let Some(song) = library.song(*song_id) {
                songs.push(song);
            }
        }
    }
    songs.sort_by_key(|song| song.id);
    songs
}

/// Comparison key of one song: decoration-stripped title, artist, and album
struct MatchKey {
    title: String,
    artist: String,
    album: String,
}

impl MatchKey {
    fn new(song: &Song, library: &Library) -> Self {
        Self {
            title: base_text(&song.title),
            artist: base_text(
                library
                    .artist(song.artist)
                    .map_or("", |artist| artist.name.as_str()),
            ),
            album: base_text(
                library
                    .album(song.album)
                    .map_or("", |album| album.title.as_str()),
            ),
        }
    }

    fn matches(&self, other: &Self) -> bool {
        similar(&self.title, &other.title)
            && similar(&self.artist, &other.artist)
            && similar(&self.album, &other.album)
    }
}

fn similar(left: &str, right: &str) -> bool {
    normalized_damerau_levenshtein(left, right) >= SIMILARITY_THRESHOLD
}

/// Turns the union-find groups into one finding per cluster of two or more songs
fn findings(candidates: &[&Song], groups: &mut DisjointSet) -> Vec<Finding> {
    let mut clusters: BTreeMap<usize, Vec<&Song>> = BTreeMap::new();
    for (index, song) in candidates.iter().copied().enumerate() {
        clusters.entry(groups.find(index)).or_default().push(song);
    }
    clusters
        .into_values()
        .filter(|songs| songs.len() > 1)
        .map(|songs| {
            Finding::AlternativeVersions(AlternativeVersionsFinding {
                songs: songs.into_iter().map(|song| song.id).collect(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
