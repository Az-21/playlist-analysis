use super::*;
use crate::parser::apple_music::xml::raw::{RawPlaylist, RawPlaylistItem, RawTrack};

fn track(name: &str, artist: &str, album: &str) -> RawTrack {
    RawTrack {
        name: Some(name.to_owned()),
        artist: Some(artist.to_owned()),
        album_artist: None,
        album: Some(album.to_owned()),
        total_time_ms: Some(180_000),
        track_number: Some(1),
        disc_number: Some(1),
    }
}

fn item(track_id: i64) -> RawPlaylistItem {
    RawPlaylistItem {
        track_id: Some(track_id),
    }
}

fn playlist(id: i64, name: &str, items: Vec<RawPlaylistItem>) -> RawPlaylist {
    RawPlaylist {
        id: Some(id),
        name: Some(name.to_owned()),
        folder: false,
        items,
    }
}

fn convert_tracks(tracks: Vec<(u64, RawTrack)>) -> (Library, ParseReport) {
    let map = tracks
        .into_iter()
        .map(|(id, track)| (id.to_string(), track))
        .collect();
    convert(map, Vec::new())
}

#[test]
fn interns_songs_albums_and_artists() {
    let (library, report) = convert_tracks(vec![
        (1, track("Alpha", "Band", "Album XYZ")),
        (2, track("Beta", "Band", "Album XYZ")),
    ]);
    assert_eq!(report, ParseReport::default());
    assert_eq!(library.songs.len(), 2);
    assert_eq!(library.albums.len(), 1);
    assert_eq!(library.artists.len(), 1);
    let first = library.song(SongId(1)).expect("song 1 exists");
    let second = library.song(SongId(2)).expect("song 2 exists");
    assert_eq!(first.album, second.album);
    assert_eq!(first.artist, second.artist);
}

#[test]
fn falls_back_to_track_artist_when_album_artist_is_missing() {
    let (library, _) = convert_tracks(vec![(1, track("Alpha", "Solo", "Album XYZ"))]);
    let album = library.album(AlbumId(0)).expect("album exists");
    let artist = library.artist(album.artist).expect("artist exists");
    assert_eq!(artist.name, "Solo");
}

#[test]
fn prefers_album_artist_for_the_album() {
    let mut raw = track("Alpha", "Track Artist", "Album XYZ");
    raw.album_artist = Some("Album Artist".to_owned());
    let (library, _) = convert_tracks(vec![(1, raw)]);
    let album = library.album(AlbumId(0)).expect("album exists");
    assert_eq!(
        library.artist(album.artist).expect("artist exists").name,
        "Album Artist"
    );
    assert_eq!(
        library
            .artist(library.song(SongId(1)).expect("song exists").artist)
            .expect("artist exists")
            .name,
        "Track Artist"
    );
}

#[test]
fn interns_case_and_whitespace_variants_into_one_entity() {
    let (library, _) = convert_tracks(vec![
        (1, track("Alpha", "The   Band", "Album XYZ")),
        (2, track("Beta", "the band", "album xyz")),
    ]);
    assert_eq!(library.artists.len(), 1);
    assert_eq!(library.albums.len(), 1);
    assert_eq!(
        library.artist(ArtistId(0)).expect("artist exists").name,
        "The   Band"
    );
    assert_eq!(
        library.album(AlbumId(0)).expect("album exists").title,
        "Album XYZ"
    );
}

#[test]
fn skips_tracks_missing_required_metadata() {
    let missing_album = RawTrack {
        album: None,
        ..track("Alpha", "Band", "Album XYZ")
    };
    let missing_name = RawTrack {
        name: None,
        ..track("Beta", "Band", "Album XYZ")
    };
    let (library, report) = convert_tracks(vec![
        (1, missing_album),
        (2, missing_name),
        (3, track("Gamma", "Band", "Album XYZ")),
    ]);
    assert_eq!(report.skipped_tracks, 2);
    assert_eq!(library.songs.len(), 1);
}

#[test]
fn skips_tracks_with_malformed_dict_keys() {
    let tracks = [(
        "not-a-number".to_owned(),
        track("Alpha", "Band", "Album XYZ"),
    )]
    .into_iter()
    .collect();
    let (library, report) = convert(tracks, Vec::new());
    assert_eq!(report.skipped_tracks, 1);
    assert_eq!(library.songs, BTreeMap::new());
}

#[test]
fn keeps_playlist_order_and_duplicates() {
    let (library, report) = convert(
        [(1_u64.to_string(), track("Alpha", "Band", "Album XYZ"))]
            .into_iter()
            .collect(),
        vec![playlist(10, "Mix", vec![item(1), item(1)])],
    );
    assert_eq!(report, ParseReport::default());
    let playlist = library.playlist(PlaylistId(10)).expect("playlist exists");
    assert_eq!(playlist.entries, vec![SongId(1), SongId(1)]);
}

#[test]
fn skips_playlist_items_referencing_unknown_tracks() {
    let (library, report) = convert(
        [(1_u64.to_string(), track("Alpha", "Band", "Album XYZ"))]
            .into_iter()
            .collect(),
        vec![playlist(10, "Mix", vec![item(1), item(999)])],
    );
    assert_eq!(report.skipped_playlist_entries, 1);
    let playlist = library.playlist(PlaylistId(10)).expect("playlist exists");
    assert_eq!(playlist.entries, vec![SongId(1)]);
}

#[test]
fn skips_playlists_without_metadata_and_empty_playlists() {
    let nameless = RawPlaylist {
        id: Some(11),
        name: None,
        folder: false,
        items: vec![item(1)],
    };
    let idless = RawPlaylist {
        id: None,
        name: Some("Idless".to_owned()),
        folder: false,
        items: vec![item(1)],
    };
    let empty = RawPlaylist {
        id: Some(12),
        name: Some("Empty".to_owned()),
        folder: false,
        items: Vec::new(),
    };
    let (library, report) = convert(
        [(1_u64.to_string(), track("Alpha", "Band", "Album XYZ"))]
            .into_iter()
            .collect(),
        vec![nameless, idless, empty],
    );
    assert_eq!(report.skipped_playlists, 2);
    assert_eq!(library.playlists, Vec::new());
}

#[test]
fn ignores_folders() {
    let folder_with_items = RawPlaylist {
        id: Some(12),
        name: Some("Folder".to_owned()),
        folder: true,
        items: vec![item(1)],
    };
    let nameless_folder = RawPlaylist {
        id: Some(13),
        name: None,
        folder: true,
        items: Vec::new(),
    };
    let (library, report) = convert(
        [(1_u64.to_string(), track("Alpha", "Band", "Album XYZ"))]
            .into_iter()
            .collect(),
        vec![folder_with_items, nameless_folder],
    );
    assert_eq!(report, ParseReport::default());
    assert_eq!(library.playlists, Vec::new());
}

#[test]
fn parses_optional_track_metadata() {
    let raw = RawTrack {
        total_time_ms: Some(209_051),
        track_number: Some(3),
        disc_number: Some(2),
        ..track("Alpha", "Band", "Album XYZ")
    };
    let (library, _) = convert_tracks(vec![(1, raw)]);
    let song = library.song(SongId(1)).expect("song exists");
    assert_eq!(song.duration_ms, Some(209_051));
    assert_eq!(song.track_number, Some(3));
    assert_eq!(song.disc_number, Some(2));
}
