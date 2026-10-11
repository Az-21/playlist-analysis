use super::*;
use crate::domain::{Album, AlbumId, Artist, ArtistId, Playlist, PlaylistId, SongId};

fn artist(id: u32, name: &str) -> Artist {
    Artist {
        id: ArtistId(id),
        name: name.to_owned(),
    }
}

fn album(id: u32, title: &str) -> Album {
    Album {
        id: AlbumId(id),
        title: title.to_owned(),
        artist: ArtistId(0),
    }
}

fn song(id: u64, title: &str, artist_id: u32, album_id: u32) -> Song {
    Song {
        id: SongId(id),
        title: title.to_owned(),
        artist: ArtistId(artist_id),
        album: AlbumId(album_id),
        duration_ms: None,
        track_number: None,
        disc_number: None,
    }
}

fn playlist(id: u64, entries: &[u64]) -> Playlist {
    Playlist {
        id: PlaylistId(id),
        name: format!("Playlist {id}"),
        entries: entries.iter().copied().map(SongId).collect(),
    }
}

fn library(
    songs: Vec<Song>,
    artists: Vec<Artist>,
    albums: Vec<Album>,
    playlists: Vec<Playlist>,
) -> Library {
    Library {
        artists: artists
            .into_iter()
            .map(|artist| (artist.id, artist))
            .collect(),
        albums: albums.into_iter().map(|album| (album.id, album)).collect(),
        songs: songs.into_iter().map(|song| (song.id, song)).collect(),
        playlists,
    }
}

fn analyze(library: &Library) -> Vec<Finding> {
    AlternativeVersions.analyze(library)
}

fn group(findings: &[Finding]) -> &AlternativeVersionsFinding {
    let [Finding::AlternativeVersions(group)] = findings else {
        panic!("expected one alternative versions finding, got {findings:?}");
    };
    group
}

#[test]
fn groups_titles_differing_only_in_decorations() {
    let library = library(
        vec![
            song(1, "Alpha", 0, 0),
            song(2, "Alpha (Remastered 2011)", 0, 0),
        ],
        vec![artist(0, "Band")],
        vec![album(0, "Album XYZ")],
        vec![playlist(10, &[1]), playlist(11, &[2])],
    );
    let findings = analyze(&library);
    assert_eq!(group(&findings).songs, vec![SongId(1), SongId(2)]);
}

#[test]
fn groups_fuzzy_artist_and_album_spellings() {
    let library = library(
        vec![song(1, "Alpha", 0, 0), song(2, "alpha", 1, 1)],
        vec![artist(0, "The Band"), artist(1, "The Band!")],
        vec![album(0, "Album XYZ"), album(1, "Album XYZ (Deluxe)")],
        vec![playlist(10, &[1, 2])],
    );
    let findings = analyze(&library);
    assert_eq!(group(&findings).songs, vec![SongId(1), SongId(2)]);
}

#[test]
fn ignores_unrelated_songs() {
    let library = library(
        vec![song(1, "Alpha", 0, 0), song(2, "Zulu", 0, 0)],
        vec![artist(0, "Band")],
        vec![album(0, "Album XYZ")],
        vec![playlist(10, &[1, 2])],
    );
    assert_eq!(analyze(&library), Vec::new());
}

#[test]
fn ignores_different_artists() {
    let library = library(
        vec![song(1, "Alpha", 0, 0), song(2, "Alpha", 1, 0)],
        vec![artist(0, "Band"), artist(1, "Solo")],
        vec![album(0, "Album XYZ")],
        vec![playlist(10, &[1, 2])],
    );
    assert_eq!(analyze(&library), Vec::new());
}

#[test]
fn ignores_dissimilar_albums() {
    let library = library(
        vec![song(1, "Alpha", 0, 0), song(2, "Alpha", 0, 1)],
        vec![artist(0, "Band")],
        vec![album(0, "Album XYZ"), album(1, "Completely Unrelated")],
        vec![playlist(10, &[1, 2])],
    );
    assert_eq!(analyze(&library), Vec::new());
}

#[test]
fn ignores_songs_that_appear_in_no_playlist() {
    let library = library(
        vec![song(1, "Alpha", 0, 0), song(2, "Alpha", 0, 0)],
        vec![artist(0, "Band")],
        vec![album(0, "Album XYZ")],
        vec![playlist(10, &[1])],
    );
    assert_eq!(analyze(&library), Vec::new());
}

#[test]
fn clusters_matches_transitively() {
    let library = library(
        vec![
            song(1, "ABCDEFG", 0, 0),
            song(2, "ABCDEFH", 0, 0),
            song(3, "ABCDEFI", 0, 0),
        ],
        vec![artist(0, "Band")],
        vec![album(0, "Album XYZ")],
        vec![playlist(10, &[1, 2, 3])],
    );
    let findings = analyze(&library);
    assert_eq!(
        group(&findings).songs,
        vec![SongId(1), SongId(2), SongId(3)]
    );
}
