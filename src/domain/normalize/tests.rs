use super::normalize_text;

#[test]
fn trims_and_collapses_whitespace_runs() {
    assert_eq!(normalize_text("  hey   jude \t now  "), "hey jude now");
}

#[test]
fn lowercases_ascii() {
    assert_eq!(normalize_text("HEY Jude"), "hey jude");
}

#[test]
fn lowercases_non_ascii() {
    assert_eq!(
        normalize_text("B\u{c9}ND  \u{c6}THER"),
        "b\u{e9}nd \u{e6}ther"
    );
}

#[test]
fn whitespace_only_input_becomes_empty() {
    assert_eq!(normalize_text(" \t\n "), "");
}

#[test]
fn already_normalized_input_is_unchanged() {
    assert_eq!(normalize_text("album xyz"), "album xyz");
}
