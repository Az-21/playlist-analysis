use super::*;
use crate::analysis::Report;

#[test]
fn exports_index_html_into_the_output_directory() {
    let dir = tempfile::tempdir().expect("temp dir");
    let output_dir = dir.path().join("report");
    let meta = ReportMeta {
        title: "Playlist Analysis".to_owned(),
        source_name: "sample.xml".to_owned(),
        generated_at: "2026-01-01T00:00:00Z".to_owned(),
    };
    HtmlExporter
        .export(
            &Report {
                findings: Vec::new(),
                total_findings: 0,
            },
            &Library::default(),
            &meta,
            &output_dir,
        )
        .expect("export succeeds");

    let html = std::fs::read_to_string(output_dir.join("index.html")).expect("index.html exists");
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Playlist Analysis</title>"));
}
