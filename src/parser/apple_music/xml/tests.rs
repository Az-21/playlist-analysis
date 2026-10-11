use super::*;
use crate::domain::{AlbumId, ArtistId, PlaylistId, SongId};
use crate::parser::ParseError;

const SAMPLE: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>Tracks</key>
	<dict>
		<key>1</key>
		<dict>
			<key>Track ID</key><integer>1</integer>
			<key>Name</key><string>Alpha</string>
			<key>Artist</key><string>Band</string>
			<key>Album</key><string>Album XYZ</string>
			<key>Total Time</key><integer>180000</integer>
		</dict>
		<key>2</key>
		<dict>
			<key>Track ID</key><integer>2</integer>
			<key>Name</key><string>Beta</string>
			<key>Artist</key><string>Band</string>
			<key>Album</key><string>Album XYZ</string>
		</dict>
	</dict>
	<key>Playlists</key>
	<array>
		<dict>
			<key>Name</key><string>Mix</string>
			<key>Playlist ID</key><integer>10</integer>
			<key>Playlist Items</key>
			<array>
				<dict><key>Track ID</key><integer>2</integer></dict>
				<dict><key>Track ID</key><integer>1</integer></dict>
				<dict><key>Track ID</key><integer>999</integer></dict>
			</array>
		</dict>
		<dict>
			<key>Name</key><string>Folder</string>
			<key>Playlist ID</key><integer>11</integer>
			<key>Folder</key><true/>
			<key>Playlist Items</key>
			<array>
				<dict><key>Track ID</key><integer>1</integer></dict>
			</array>
		</dict>
	</array>
</dict>
</plist>"#;

#[test]
fn parses_tracks_playlists_and_reports() {
    let output = parse(SAMPLE).expect("sample parses");
    assert_eq!(output.report.skipped_tracks, 0);
    assert_eq!(output.report.skipped_playlist_entries, 1);
    assert_eq!(output.library.songs.len(), 2);
    assert_eq!(output.library.albums.len(), 1);
    assert_eq!(output.library.artists.len(), 1);
    assert_eq!(output.library.playlists.len(), 1);
    let playlist = output
        .library
        .playlist(PlaylistId(10))
        .expect("playlist exists");
    assert_eq!(playlist.entries, vec![SongId(2), SongId(1)]);
    assert_eq!(
        output
            .library
            .album(AlbumId(0))
            .expect("album exists")
            .title,
        "Album XYZ"
    );
    assert_eq!(
        output
            .library
            .artist(ArtistId(0))
            .expect("artist exists")
            .name,
        "Band"
    );
}

#[test]
fn rejects_malformed_xml() {
    let error = parse(b"<plist><dict>").expect_err("malformed XML is rejected");
    assert!(matches!(error, ParseError::InvalidPlist(_)));
}

#[test]
fn rejects_missing_sections() {
    let missing_tracks = br#"<?xml version="1.0"?><plist version="1.0"><dict><key>Playlists</key><array/></dict></plist>"#;
    let error = parse(missing_tracks).expect_err("tracks are required");
    assert!(matches!(error, ParseError::MissingSection("Tracks")));

    let missing_playlists = br#"<?xml version="1.0"?><plist version="1.0"><dict><key>Tracks</key><dict/></dict></plist>"#;
    let error = parse(missing_playlists).expect_err("playlists are required");
    assert!(matches!(error, ParseError::MissingSection("Playlists")));
}
