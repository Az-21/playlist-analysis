//! Fuzzy detection of alternative versions of the same song

use std::collections::{BTreeMap, BTreeSet};

use strsim::normalized_damerau_levenshtein;

use crate::analysis::Analysis;
use crate::analysis::report::AlternativeVersions as AlternativeVersionsFinding;
use crate::analysis::report::Finding;
use crate::domain::{Library, Song, normalize_text};

/// Minimum similarity for two songs to count as possible alternative versions
const SIMILARITY_THRESHOLD: f64 = 0.5;

/// Keywords that mark a bracketed or dashed segment as a decoration
const DECORATION_KEYWORDS: &[&str] = &[
    "acoustic",
    "anniversary",
    "bonus",
    "deluxe",
    "demo",
    "edit",
    "edition",
    "explicit",
    "extended",
    "feat",
    "ft",
    "instrumental",
    "live",
    "mix",
    "mono",
    "radio",
    "remaster",
    "remix",
    "reprise",
    "session",
    "single",
    "stereo",
    "take",
    "version",
];

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

/// Lowercases, collapses whitespace, and drops decoration segments such as
/// `(Remastered 2011)`, `[Deluxe Edition]`, `- Live`, or `feat. X`
fn base_text(input: &str) -> String {
    normalize_text(&strip_decorations(&normalize_text(input)))
}

fn strip_decorations(input: &str) -> String {
    let mut stripped = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(open) = rest.find(['(', '[']) {
        let (before, from_open) = rest.split_at(open);
        stripped.push_str(before);
        let close = if from_open.starts_with('(') { ')' } else { ']' };
        let Some(end) = from_open.find(close) else {
            stripped.push_str(from_open);
            rest = "";
            break;
        };
        if !contains_decoration(&from_open[1..end]) {
            stripped.push_str(&from_open[..=end]);
        }
        rest = &from_open[end + 1..];
    }
    stripped.push_str(rest);

    let kept: Vec<&str> = stripped
        .split(" - ")
        .enumerate()
        .filter(|(index, segment)| *index == 0 || !contains_decoration(segment))
        .map(|(_, segment)| segment)
        .collect();
    let dashed = kept.join(" - ");

    let stripped = match dashed.find(" feat") {
        Some(position) => &dashed[..position],
        None => &dashed,
    };
    stripped.trim().to_owned()
}

fn contains_decoration(segment: &str) -> bool {
    segment
        .split(|character: char| !character.is_alphanumeric())
        .any(|token| {
            DECORATION_KEYWORDS
                .iter()
                .any(|keyword| token.starts_with(keyword))
        })
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

/// Union-find over candidate indices
struct DisjointSet {
    parents: Vec<usize>,
}

impl DisjointSet {
    fn new(len: usize) -> Self {
        Self {
            parents: (0..len).collect(),
        }
    }

    fn find(&mut self, index: usize) -> usize {
        let mut root = index;
        while self.parents[root] != root {
            root = self.parents[root];
        }
        let mut current = index;
        while self.parents[current] != root {
            let next = self.parents[current];
            self.parents[current] = root;
            current = next;
        }
        root
    }

    fn union(&mut self, left: usize, right: usize) {
        let left_root = self.find(left);
        let right_root = self.find(right);
        if left_root != right_root {
            self.parents[right_root] = left_root;
        }
    }
}

#[cfg(test)]
mod tests;
