use super::*;
use crate::domain::{Playlist, PlaylistId, Song};

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

fn analyze(library: &Library) -> Vec<Finding> {
    SameTitleDifferentAlbum.analyze(library)
}

#[test]
fn flags_the_same_song_on_a_standard_and_deluxe_album() {
    let library = library(
        vec![song(1, "Beta", 0, 0), song(2, "Beta", 0, 1)],
        vec![playlist(10, vec![SongId(1)]), playlist(11, vec![SongId(2)])],
    );
    assert_eq!(
        analyze(&library),
        vec![Finding::SameTitleDifferentAlbum(
            SameTitleDifferentAlbumFinding {
                title: "Beta".to_owned(),
                artist: ArtistId(0),
                albums: vec![AlbumId(0), AlbumId(1)],
                songs: vec![SongId(1), SongId(2)],
            }
        )]
    );
}

#[test]
fn ignores_the_same_title_on_one_album() {
    let library = library(
        vec![song(1, "Beta", 0, 0), song(2, "Beta", 0, 0)],
        vec![playlist(10, vec![SongId(1), SongId(2)])],
    );
    assert_eq!(analyze(&library), Vec::new());
}

#[test]
fn ignores_the_same_title_by_different_artists() {
    let library = library(
        vec![song(1, "Beta", 0, 0), song(2, "Beta", 1, 1)],
        vec![playlist(10, vec![SongId(1), SongId(2)])],
    );
    assert_eq!(analyze(&library), Vec::new());
}

#[test]
fn matches_titles_and_artists_across_case_and_whitespace() {
    let library = library(
        vec![song(1, "Beta", 0, 0), song(2, "  beta ", 0, 1)],
        vec![playlist(10, vec![SongId(1), SongId(2)])],
    );
    assert_eq!(analyze(&library).len(), 1);
}

#[test]
fn ignores_songs_that_appear_in_no_playlist() {
    let library = library(
        vec![song(1, "Beta", 0, 0), song(2, "Beta", 0, 1)],
        vec![playlist(10, vec![SongId(1)])],
    );
    assert_eq!(analyze(&library), Vec::new());
}

#[test]
fn counts_each_song_once_even_when_referenced_by_many_playlists() {
    let library = library(
        vec![song(1, "Beta", 0, 0), song(2, "Beta", 0, 1)],
        vec![
            playlist(10, vec![SongId(1), SongId(2)]),
            playlist(11, vec![SongId(1), SongId(2), SongId(2)]),
        ],
    );
    let findings = analyze(&library);
    assert_eq!(findings.len(), 1);
    let Finding::SameTitleDifferentAlbum(finding) = &findings[0] else {
        panic!("expected album variant finding, got {:?}", findings[0]);
    };
    assert_eq!(finding.songs, vec![SongId(1), SongId(2)]);
}
