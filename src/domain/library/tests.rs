use super::*;
use crate::domain::{Artist, Playlist, Song};

fn library_with_song() -> (Library, SongId) {
    let artist = Artist {
        id: ArtistId(0),
        name: "Band".to_owned(),
    };
    let album = Album {
        id: AlbumId(0),
        title: "Album XYZ".to_owned(),
        artist: ArtistId(0),
    };
    let song = Song {
        id: SongId(7),
        title: "Alpha".to_owned(),
        artist: ArtistId(0),
        album: AlbumId(0),
        duration_ms: Some(65_000),
        track_number: Some(1),
        disc_number: Some(1),
    };
    let playlist = Playlist {
        id: PlaylistId(3),
        name: "Mix".to_owned(),
        entries: vec![SongId(7)],
    };
    let library = Library {
        artists: [(ArtistId(0), artist)].into(),
        albums: [(AlbumId(0), album)].into(),
        songs: [(SongId(7), song.clone())].into(),
        playlists: vec![playlist],
    };
    (library, SongId(7))
}

#[test]
fn lookups_resolve_known_ids() {
    let (library, song_id) = library_with_song();
    let song = library.song(song_id).expect("song was inserted");
    assert_eq!(song.title, "Alpha");
    assert_eq!(
        library.album(song.album).expect("album was inserted").title,
        "Album XYZ"
    );
    assert_eq!(
        library
            .artist(song.artist)
            .expect("artist was inserted")
            .name,
        "Band"
    );
    assert_eq!(
        library
            .playlist(PlaylistId(3))
            .expect("playlist was inserted")
            .entries,
        vec![song_id]
    );
}

#[test]
fn lookups_return_none_for_unknown_ids() {
    let (library, _) = library_with_song();
    assert!(library.song(SongId(999)).is_none());
    assert!(library.album(AlbumId(999)).is_none());
    assert!(library.artist(ArtistId(999)).is_none());
    assert!(library.playlist(PlaylistId(999)).is_none());
}

#[test]
fn formats_short_durations() {
    assert_eq!(format_duration(0), "0:00");
    assert_eq!(format_duration(59_999), "0:59");
    assert_eq!(format_duration(60_000), "1:00");
    assert_eq!(format_duration(209_051), "3:29");
}

#[test]
fn formats_hour_long_durations() {
    assert_eq!(format_duration(3_600_000), "1:00:00");
    assert_eq!(format_duration(3_661_000), "1:01:01");
}
