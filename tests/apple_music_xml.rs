use std::path::PathBuf;

use playlist_analysis::analysis::{self, Finding};
use playlist_analysis::domain::{ArtistId, PlaylistId, SongId};
use playlist_analysis::parser::{self, LibraryFormat, Service, SourceKind};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/apple_music_sample.xml")
}

fn parse_fixture() -> parser::ParseOutput {
    let input = std::fs::read(fixture_path()).expect("fixture is readable");
    let source = parser::detect(&input).expect("fixture is detected");
    assert_eq!(
        source,
        SourceKind {
            service: Service::AppleMusic,
            format: LibraryFormat::Xml
        }
    );
    parser::parse(source, &input).expect("fixture parses")
}

#[test]
fn parses_tracks_albums_artists_and_playlists() {
    let output = parse_fixture();
    assert_eq!(output.report.skipped_tracks, 1);
    assert_eq!(output.report.skipped_playlist_entries, 1);
    assert_eq!(output.report.skipped_playlists, 1);
    assert_eq!(output.library.songs.len(), 7);
    assert_eq!(output.library.albums.len(), 4);
    assert_eq!(output.library.artists.len(), 3);
    assert_eq!(output.library.playlists.len(), 3);

    let road_trip = output
        .library
        .playlist(PlaylistId(100))
        .expect("playlist exists");
    assert_eq!(
        road_trip.entries,
        vec![SongId(1), SongId(5), SongId(2), SongId(8)]
    );
    let broken = output
        .library
        .playlist(PlaylistId(102))
        .expect("playlist exists");
    assert_eq!(broken.entries, vec![SongId(1)]);
    let alpha = output.library.song(SongId(1)).expect("song exists");
    assert_eq!(alpha.duration_ms, Some(185_000));
    assert_eq!(alpha.track_number, Some(1));
    assert_eq!(alpha.disc_number, Some(1));
    assert_eq!(
        output
            .library
            .album(alpha.album)
            .expect("album exists")
            .title,
        "Album XYZ"
    );
    assert_eq!(
        output
            .library
            .artist(alpha.artist)
            .expect("artist exists")
            .name,
        "Band"
    );
}

#[test]
fn interns_unicode_metadata() {
    let output = parse_fixture();
    let eclair = output.library.song(SongId(7)).expect("song exists");
    assert_eq!(eclair.title, "\u{c9}clair");
    assert_eq!(
        output
            .library
            .album(eclair.album)
            .expect("album exists")
            .title,
        "\u{410}\u{43b}\u{44c}\u{431}\u{43e}\u{43c}"
    );
    assert_eq!(
        output
            .library
            .artist(eclair.artist)
            .expect("artist exists")
            .name,
        "\u{411}\u{44d}\u{43d}\u{434}"
    );
}

#[test]
fn analyzes_duplicates_album_variants_alternative_versions_and_similar_albums() {
    let output = parse_fixture();
    let report = analysis::run(&output.library, &analysis::default_analyses());
    assert_eq!(report.total_findings, 5);

    let duplicate = report
        .findings
        .iter()
        .find_map(|finding| match finding {
            Finding::DuplicateEntries(duplicate) => Some(duplicate),
            Finding::SameTitleDifferentAlbum(_)
            | Finding::AlternativeVersions(_)
            | Finding::SimilarAlbums(_) => None,
        })
        .expect("duplicate finding exists");
    assert_eq!(duplicate.playlist, PlaylistId(100));
    assert_eq!(duplicate.title, "Alpha");
    assert_eq!(duplicate.occurrences.len(), 2);

    let variant = report
        .findings
        .iter()
        .find_map(|finding| match finding {
            Finding::SameTitleDifferentAlbum(variant) => Some(variant),
            Finding::DuplicateEntries(_)
            | Finding::AlternativeVersions(_)
            | Finding::SimilarAlbums(_) => None,
        })
        .expect("album variant finding exists");
    assert_eq!(variant.title, "Beta");
    assert_eq!(variant.artist, ArtistId(0));
    assert_eq!(variant.songs, vec![SongId(2), SongId(3)]);
    assert_eq!(variant.albums.len(), 2);
    assert_eq!(
        output
            .library
            .album(variant.albums[1])
            .expect("album exists")
            .title,
        "Album XYZ (Deluxe)"
    );

    let alternatives: Vec<_> = report
        .findings
        .iter()
        .filter_map(|finding| match finding {
            Finding::AlternativeVersions(alternative) => Some(alternative),
            Finding::DuplicateEntries(_)
            | Finding::SameTitleDifferentAlbum(_)
            | Finding::SimilarAlbums(_) => None,
        })
        .collect();
    assert_eq!(alternatives.len(), 2);
    assert_eq!(alternatives[0].songs, vec![SongId(1), SongId(5), SongId(8)]);
    assert_eq!(alternatives[1].songs, vec![SongId(2), SongId(3)]);

    let similar = report
        .findings
        .iter()
        .find_map(|finding| match finding {
            Finding::SimilarAlbums(similar) => Some(similar),
            Finding::DuplicateEntries(_)
            | Finding::SameTitleDifferentAlbum(_)
            | Finding::AlternativeVersions(_) => None,
        })
        .expect("similar albums finding exists");
    let titles: Vec<&str> = similar
        .albums
        .iter()
        .map(|album| {
            output
                .library
                .album(*album)
                .expect("album exists")
                .title
                .as_str()
        })
        .collect();
    assert_eq!(titles, vec!["Album XYZ", "Album XYZ (Deluxe)"]);
}
