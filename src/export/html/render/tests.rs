use super::*;
use crate::analysis::{DuplicateOccurrence, Report};
use crate::domain::{Album, Artist, Playlist, PlaylistId, Song};

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

fn song(id: u64, title: &str, album_id: u32, duration_ms: Option<u64>) -> Song {
    Song {
        id: SongId(id),
        title: title.to_owned(),
        artist: ArtistId(0),
        album: AlbumId(album_id),
        duration_ms,
        track_number: None,
        disc_number: None,
    }
}

fn playlist(id: u64, name: &str, entries: Vec<SongId>) -> Playlist {
    Playlist {
        id: PlaylistId(id),
        name: name.to_owned(),
        entries,
    }
}

fn meta() -> ReportMeta {
    ReportMeta {
        title: "Test Report".to_owned(),
        source_name: "sample.xml".to_owned(),
        generated_at: "2026-01-01T00:00:00Z".to_owned(),
    }
}

fn fixture_library() -> Library {
    Library {
        artists: [artist(0, "Band")]
            .into_iter()
            .map(|artist| (artist.id, artist))
            .collect(),
        albums: [album(0, "Album XYZ"), album(1, "Album XYZ (Deluxe)")]
            .into_iter()
            .map(|album| (album.id, album))
            .collect(),
        songs: [
            song(1, "Alpha", 0, Some(185_000)),
            song(2, "Beta", 0, Some(200_000)),
            song(3, "Beta", 1, None),
            song(4, "Alpha", 0, None),
        ]
        .into_iter()
        .map(|song| (song.id, song))
        .collect(),
        playlists: vec![
            playlist(10, "Mix", vec![SongId(1), SongId(4), SongId(2)]),
            playlist(11, "Deluxe Mix", vec![SongId(3)]),
        ],
    }
}

fn fixture_report() -> Report {
    Report {
        findings: vec![
            Finding::DuplicateEntries(DuplicateEntries {
                playlist: PlaylistId(10),
                title: "Alpha".to_owned(),
                artist: ArtistId(0),
                album: AlbumId(0),
                occurrences: vec![
                    DuplicateOccurrence {
                        position: 0,
                        song: SongId(1),
                    },
                    DuplicateOccurrence {
                        position: 1,
                        song: SongId(4),
                    },
                ],
            }),
            Finding::SameTitleDifferentAlbum(SameTitleDifferentAlbum {
                title: "Beta".to_owned(),
                artist: ArtistId(0),
                albums: vec![AlbumId(0), AlbumId(1)],
                songs: vec![SongId(2), SongId(3)],
            }),
            Finding::AlternativeVersions(AlternativeVersions {
                songs: vec![SongId(1), SongId(4)],
            }),
            Finding::AlternativeVersions(AlternativeVersions {
                songs: vec![SongId(2), SongId(3)],
            }),
            Finding::SimilarAlbums(SimilarAlbums {
                albums: vec![AlbumId(0), AlbumId(1)],
            }),
        ],
        total_findings: 5,
    }
}

#[test]
fn renders_header_summary_and_findings() {
    let html = render(&fixture_report(), &fixture_library(), &meta());
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Test Report</title>"));
    assert!(html.contains("Source: <span class=\"mono\">sample.xml</span>"));
    assert!(html.contains("Library: 4 songs, 2 albums, 1 artists, 2 playlists"));
    assert!(html.contains("Playlists analyzed"));
    assert!(html.contains("Duplicate findings"));
    assert!(html.contains("Similar albums"));
}

#[test]
fn renders_one_based_positions_and_track_ids() {
    let html = render(&fixture_report(), &fixture_library(), &meta());
    assert!(html.contains("<td class=\"mono\">1, 2</td>"));
    assert!(html.contains("<td class=\"mono\">1, 4</td>"));
    assert!(html.contains("<h3>Mix</h3>"));
}

#[test]
fn renders_album_variants_with_their_playlists() {
    let html = render(&fixture_report(), &fixture_library(), &meta());
    assert!(html.contains("Same Title, Different Album"));
    assert!(html.contains("<li>Album XYZ<ul class=\"tight\"><li>Mix</li></ul></li>"));
    assert!(
        html.contains("<li>Album XYZ (Deluxe)<ul class=\"tight\"><li>Deluxe Mix</li></ul></li>")
    );
}

#[test]
fn groups_playlists_under_each_album() {
    let mut library = fixture_library();
    library.playlists = vec![playlist(10, "Mix", vec![SongId(2), SongId(3)])];
    let report = Report {
        findings: vec![Finding::SameTitleDifferentAlbum(SameTitleDifferentAlbum {
            title: "Beta".to_owned(),
            artist: ArtistId(0),
            albums: vec![AlbumId(0), AlbumId(1)],
            songs: vec![SongId(2), SongId(3)],
        })],
        total_findings: 1,
    };
    let html = render(&report, &library, &meta());
    assert!(html.contains("<li>Album XYZ<ul class=\"tight\"><li>Mix</li></ul></li>"));
    assert!(html.contains("<li>Album XYZ (Deluxe)<ul class=\"tight\"><li>Mix</li></ul></li>"));
}

#[test]
fn renders_alternative_versions_as_grouped_variant_rows() {
    let html = render(&fixture_report(), &fixture_library(), &meta());
    assert!(html.contains("<h2>Alternative Versions</h2>"));
    assert!(html.contains(
        "<tr><td>Alpha</td><td>Band</td><td>Album XYZ</td><td><ul class=\"tight\"><li>Mix</li></ul></td></tr>"
    ));
    assert!(html.contains(
        "<tr class=\"group\"><td>Beta</td><td>Band</td><td>Album XYZ</td><td><ul class=\"tight\"><li>Mix</li></ul></td></tr>"
    ));
    assert!(html.contains(
        "<tr><td>Beta</td><td>Band</td><td>Album XYZ (Deluxe)</td><td><ul class=\"tight\"><li>Deluxe Mix</li></ul></td></tr>"
    ));
}

#[test]
fn renders_similar_albums_with_their_playlists() {
    let html = render(&fixture_report(), &fixture_library(), &meta());
    assert!(html.contains("<h2>Similar Albums</h2>"));
    assert!(html.contains(concat!(
        "<tr><td><ul class=\"tight\">",
        "<li>Album XYZ <span class=\"meta\">Band</span>",
        "<ul class=\"tight\"><li>Mix</li></ul></li>",
        "<li>Album XYZ (Deluxe) <span class=\"meta\">Band</span>",
        "<ul class=\"tight\"><li>Deluxe Mix</li></ul></li>",
        "</ul></td></tr>"
    )));
}

#[test]
fn renders_empty_states_when_there_are_no_findings() {
    let report = Report {
        findings: Vec::new(),
        total_findings: 0,
    };
    let html = render(&report, &fixture_library(), &meta());
    assert!(html.contains("No playlist contains the same song twice."));
    assert!(html.contains("No songs share a title and artist across different albums."));
    assert!(html.contains("No similar album versions were found."));
    assert!(html.contains("No alternative versions of the same song were found."));
    assert!(!html.contains("<table>"));
}

#[test]
fn escapes_metadata_instead_of_emitting_markup() {
    let mut library = fixture_library();
    let song = library.songs.get_mut(&SongId(1)).expect("song exists");
    song.title = "<script>alert('x')</script>".to_owned();
    let report = Report {
        findings: vec![Finding::DuplicateEntries(DuplicateEntries {
            playlist: PlaylistId(10),
            title: song.title.clone(),
            artist: ArtistId(0),
            album: AlbumId(0),
            occurrences: vec![
                DuplicateOccurrence {
                    position: 0,
                    song: SongId(1),
                },
                DuplicateOccurrence {
                    position: 1,
                    song: SongId(4),
                },
            ],
        })],
        total_findings: 1,
    };
    let html = render(&report, &library, &meta());
    assert!(!html.contains("<script>alert"));
    assert!(html.contains("&lt;script&gt;alert(&#39;x&#39;)&lt;/script&gt;"));
}

#[test]
fn renders_unknown_album_and_artist_placeholders() {
    let report = Report {
        findings: vec![Finding::SameTitleDifferentAlbum(SameTitleDifferentAlbum {
            title: "Ghost".to_owned(),
            artist: ArtistId(99),
            albums: vec![AlbumId(99)],
            songs: vec![SongId(99)],
        })],
        total_findings: 1,
    };
    let html = render(&report, &fixture_library(), &meta());
    assert!(html.contains("(unknown artist)"));
    assert!(html.contains("(unknown album)"));
    assert!(html.contains("<li>(none)</li>"));
}
