use playlist_analysis::analysis;
use playlist_analysis::export::{self, ReportMeta};
use playlist_analysis::parser;

#[test]
fn exports_a_deterministic_html_report() {
    let input = std::fs::read(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/apple_music_sample.xml"),
    )
    .expect("fixture is readable");
    let source = parser::detect(&input).expect("fixture is detected");
    let parsed = parser::parse(source, &input).expect("fixture parses");
    let report = analysis::run(&parsed.library, &analysis::default_analyses());

    let meta = ReportMeta {
        title: "Playlist Analysis".to_owned(),
        source_name: "apple_music_sample.xml".to_owned(),
        generated_at: "2026-01-01T00:00:00Z".to_owned(),
    };
    let output_dir = tempfile::tempdir().expect("temp dir");
    export::exporter("html")
        .expect("html exporter exists")
        .export(&report, &parsed.library, &meta, output_dir.path())
        .expect("export succeeds");

    let html =
        std::fs::read_to_string(output_dir.path().join("index.html")).expect("index.html exists");
    assert!(html.contains("<h3>Road Trip</h3>"));
    assert!(
        html.contains("<li>Album XYZ (Deluxe)<ul class=\"tight\"><li>Deluxe Mix</li></ul></li>")
    );
    insta::assert_snapshot!("html_report", html);
}
