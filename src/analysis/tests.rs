use super::*;
use crate::domain::Library;

#[test]
fn default_analyses_register_all_checks() {
    let analyses = default_analyses();
    let ids: Vec<&str> = analyses.iter().map(|analysis| analysis.id()).collect();
    assert_eq!(
        ids,
        vec![
            "duplicate-songs",
            "same-title-different-album",
            "alternative-versions",
            "similar-albums"
        ]
    );
}

#[test]
fn run_on_an_empty_library_reports_nothing() {
    let report = run(&Library::default(), &default_analyses());
    assert_eq!(report.total_findings, 0);
    assert_eq!(report.findings, Vec::new());
}
