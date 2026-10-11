use super::*;

#[test]
fn finds_the_html_exporter() {
    let exporter = exporter("html").expect("html is registered");
    assert_eq!(exporter.id(), "html");
    assert_eq!(default_exporters().len(), 1);
}

#[test]
fn rejects_unknown_exporters() {
    let error = exporter("pdf").err().expect("pdf is not registered");
    assert!(matches!(error, WriteError::UnknownFormat(id) if id == "pdf"));
}
