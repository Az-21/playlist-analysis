use super::{LibraryFormat, Service, detect};

#[test]
fn detects_apple_music_xml() {
    let input = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict><key>Tracks</key><dict/></dict></plist>"#;
    assert_eq!(
        detect(input),
        Some(super::SourceKind {
            service: Service::AppleMusic,
            format: LibraryFormat::Xml
        })
    );
}

#[test]
fn rejects_non_xml_input() {
    assert_eq!(detect(b"{\"tracks\": []}"), None);
    assert_eq!(detect(b""), None);
}

#[test]
fn detects_a_very_large_export_from_its_head() {
    let mut input = Vec::from(br#"<?xml version="1.0"?><plist version="1.0">"#);
    input.extend(std::iter::repeat_n(b'x', 50_000));
    assert!(detect(&input).is_some());
}
