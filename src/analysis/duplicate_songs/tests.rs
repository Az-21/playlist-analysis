use super::*;
use crate::domain::{PlaylistId, Song, SongId};

fn song(id: u64, title: &str, artist: u32, album: u32) -> Song {
    Song {
        id: SongId(id),
        title: title.to_owned(),
        artist: ArtistId(artist),
        album: AlbumId(album),
        duration_ms: None,
        track_number: None,
        disc_number: None,
    }
}

fn library(songs: Vec<Song>, playlists: Vec<Playlist>) -> Library {
    Library {
        artists: BTreeMap::new(),
        albums: BTreeMap::new(),
        songs: songs.into_iter().map(|song| (song.id, song)).collect(),
        playlists,
    }
}

fn playlist(id: u64, entries: Vec<SongId>) -> Playlist {
    Playlist {
        id: PlaylistId(id),
        name: format!("Playlist {id}"),
        entries,
    }
}

fn entries(ids: &[u64]) -> Vec<SongId> {
    ids.iter().copied().map(SongId).collect()
}

fn analyze(library: &Library) -> Vec<Finding> {
    DuplicateSongs.analyze(library)
}

#[test]
fn reports_nothing_for_empty_or_single_entry_playlists() {
    let library = library(
        vec![song(1, "Alpha", 0, 0)],
        vec![playlist(10, Vec::new()), playlist(11, entries(&[1]))],
    );
    assert_eq!(analyze(&library), Vec::new());
}

#[test]
fn flags_a_repeated_track_id() {
    let library = library(
        vec![song(1, "Alpha", 0, 0)],
        vec![playlist(10, entries(&[1, 1]))],
    );
    assert_eq!(
        analyze(&library),
        vec![Finding::DuplicateEntries(DuplicateEntries {
            playlist: PlaylistId(10),
            title: "Alpha".to_owned(),
            artist: ArtistId(0),
            album: AlbumId(0),
            occurrences: vec![
                DuplicateOccurrence {
                    position: 0,
                    song: SongId(1)
                },
                DuplicateOccurrence {
                    position: 1,
                    song: SongId(1)
                },
            ],
        })]
    );
}

#[test]
fn flags_distinct_ids_with_identical_metadata() {
    let library = library(
        vec![song(1, "Alpha", 0, 0), song(2, "Alpha", 0, 0)],
        vec![playlist(10, entries(&[1, 2]))],
    );
    assert_eq!(
        analyze(&library),
        vec![Finding::DuplicateEntries(DuplicateEntries {
            playlist: PlaylistId(10),
            title: "Alpha".to_owned(),
            artist: ArtistId(0),
            album: AlbumId(0),
            occurrences: vec![
                DuplicateOccurrence {
                    position: 0,
                    song: SongId(1)
                },
                DuplicateOccurrence {
                    position: 1,
                    song: SongId(2)
                },
            ],
        })]
    );
}

#[test]
fn ignores_same_title_on_different_albums() {
    let library = library(
        vec![song(1, "Alpha", 0, 0), song(2, "Alpha", 0, 1)],
        vec![playlist(10, entries(&[1, 2]))],
    );
    assert_eq!(analyze(&library), Vec::new());
}

#[test]
fn matches_titles_case_insensitively() {
    let library = library(
        vec![song(1, "Alpha", 0, 0), song(2, "  alpha ", 0, 0)],
        vec![playlist(10, entries(&[1, 2]))],
    );
    assert_eq!(analyze(&library).len(), 1);
}

#[test]
fn reports_duplicates_of_each_playlist_separately() {
    let library = library(
        vec![song(1, "Alpha", 0, 0)],
        vec![
            playlist(10, entries(&[1, 1])),
            playlist(11, entries(&[1, 1, 1])),
        ],
    );
    let findings = analyze(&library);
    assert_eq!(findings.len(), 2);
    let first = match &findings[0] {
        Finding::DuplicateEntries(duplicate) => duplicate,
        other @ (Finding::SameTitleDifferentAlbum(_)
        | Finding::AlternativeVersions(_)
        | Finding::SimilarAlbums(_)) => {
            panic!("expected duplicate entries, got {other:?}")
        }
    };
    assert_eq!(first.playlist, PlaylistId(10));
    assert_eq!(first.occurrences.len(), 2);
    let second = match &findings[1] {
        Finding::DuplicateEntries(duplicate) => duplicate,
        other @ (Finding::SameTitleDifferentAlbum(_)
        | Finding::AlternativeVersions(_)
        | Finding::SimilarAlbums(_)) => {
            panic!("expected duplicate entries, got {other:?}")
        }
    };
    assert_eq!(second.playlist, PlaylistId(11));
    assert_eq!(second.occurrences.len(), 3);
}

#[test]
fn sorts_findings_by_first_occurrence() {
    let library = library(
        vec![song(1, "Alpha", 0, 0), song(2, "Beta", 0, 0)],
        vec![playlist(10, entries(&[2, 1, 1, 2]))],
    );
    let findings = analyze(&library);
    assert_eq!(findings.len(), 2);
    let first_titles: Vec<&str> = findings
        .iter()
        .filter_map(|finding| match finding {
            Finding::DuplicateEntries(duplicate) => Some(duplicate.title.as_str()),
            Finding::SameTitleDifferentAlbum(_)
            | Finding::AlternativeVersions(_)
            | Finding::SimilarAlbums(_) => None,
        })
        .collect();
    assert_eq!(first_titles, vec!["Beta", "Alpha"]);
}
