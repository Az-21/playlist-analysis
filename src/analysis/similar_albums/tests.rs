use super::*;
use crate::domain::{Album, Artist, ArtistId, Playlist, PlaylistId, Song, SongId};

fn artist(id: u32, name: &str) -> Artist {
    Artist {
        id: ArtistId(id),
        name: name.to_owned(),
    }
}

fn album(id: u32, title: &str, artist_id: u32) -> Album {
    Album {
        id: AlbumId(id),
        title: title.to_owned(),
        artist: ArtistId(artist_id),
    }
}

fn song(id: u64, album_id: u32) -> Song {
    Song {
        id: SongId(id),
        title: format!("Track {id}"),
        artist: ArtistId(0),
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

/// One album by "Band" per title, each with a song used by its own playlist
fn library_for_titles(titles: &[&str]) -> Library {
    let albums: Vec<Album> = titles
        .iter()
        .zip(0_u32..)
        .map(|(title, id)| album(id, title, 0))
        .collect();
    let songs: Vec<Song> = albums
        .iter()
        .zip(1_u64..)
        .map(|(album, id)| song(id, album.id.0))
        .collect();
    let playlists: Vec<Playlist> = songs
        .iter()
        .zip(10_u64..)
        .map(|(song, id)| playlist(id, &[song.id.0]))
        .collect();
    library(songs, vec![artist(0, "Band")], albums, playlists)
}

fn analyze(library: &Library) -> Vec<Finding> {
    SimilarAlbums.analyze(library)
}

fn group(findings: &[Finding]) -> &SimilarAlbumsFinding {
    let [Finding::SimilarAlbums(group)] = findings else {
        panic!("expected one similar albums finding, got {findings:?}");
    };
    group
}

#[test]
fn flags_titles_that_differ_only_in_decoration_tags() {
    for decorated in [
        "Album XYZ (Deluxe)",
        "Album XYZ (Remaster 2026)",
        "Album XYZ [Deluxe Edition]",
        "Album XYZ - Live",
    ] {
        let library = library_for_titles(&["Album XYZ", decorated]);
        let findings = analyze(&library);
        assert_eq!(
            group(&findings).albums,
            vec![AlbumId(0), AlbumId(1)],
            "decorated title: {decorated}"
        );
    }
}

#[test]
fn flags_near_identical_titles_without_decoration_tags() {
    let library = library_for_titles(&["Hotel California", "Hotel Califoria"]);
    let findings = analyze(&library);
    assert_eq!(group(&findings).albums, vec![AlbumId(0), AlbumId(1)]);
}

#[test]
fn flags_similar_titles_by_different_artists() {
    let library = library(
        vec![song(1, 0), song(2, 1)],
        vec![artist(0, "Band"), artist(1, "Solo")],
        vec![album(0, "Album XYZ", 0), album(1, "Album XYZ (Deluxe)", 1)],
        vec![playlist(10, &[1]), playlist(11, &[2])],
    );
    let findings = analyze(&library);
    assert_eq!(group(&findings).albums, vec![AlbumId(0), AlbumId(1)]);
}

#[test]
fn ignores_unrelated_titles() {
    let library = library_for_titles(&["Album XYZ", "Other Album"]);
    assert_eq!(analyze(&library), Vec::new());
}

#[test]
fn ignores_albums_that_appear_in_no_playlist() {
    let mut library = library_for_titles(&["Album XYZ", "Album XYZ (Deluxe)"]);
    library
        .playlists
        .retain(|playlist| playlist.id == PlaylistId(10));
    assert_eq!(analyze(&library), Vec::new());
}

#[test]
fn clusters_variants_transitively() {
    let library = library_for_titles(&["ABCDEFG", "ABCDEFH", "ABCDEFI"]);
    let findings = analyze(&library);
    assert_eq!(
        group(&findings).albums,
        vec![AlbumId(0), AlbumId(1), AlbumId(2)]
    );
}

#[test]
fn reports_each_group_separately() {
    let library = library_for_titles(&[
        "Album XYZ",
        "Album XYZ (Deluxe)",
        "Other Album",
        "Other Album (Remaster 2026)",
    ]);
    let findings = analyze(&library);
    let groups: Vec<Vec<AlbumId>> = findings
        .iter()
        .map(|finding| {
            let Finding::SimilarAlbums(group) = finding else {
                panic!("expected similar albums finding, got {finding:?}");
            };
            group.albums.clone()
        })
        .collect();
    assert_eq!(
        groups,
        vec![vec![AlbumId(0), AlbumId(1)], vec![AlbumId(2), AlbumId(3)]]
    );
}

#[test]
fn ignores_sequels_that_share_a_prefix() {
    let library = library_for_titles(&["Greatest Hits", "Greatest Hits Vol 2"]);
    assert_eq!(analyze(&library), Vec::new());
}

#[test]
fn ignores_titles_that_strip_to_nothing() {
    let library = library_for_titles(&["(Deluxe)", "(Remastered 2011)"]);
    assert_eq!(analyze(&library), Vec::new());
}
