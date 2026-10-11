use super::{base_text, strip_decorations};

#[test]
fn strips_bracketed_and_dashed_decorations() {
    assert_eq!(strip_decorations("alpha (remastered 2011)"), "alpha");
    assert_eq!(strip_decorations("alpha [deluxe edition]"), "alpha");
    assert_eq!(strip_decorations("alpha - live"), "alpha");
    assert_eq!(strip_decorations("alpha - 2011 remaster"), "alpha");
    assert_eq!(strip_decorations("alpha feat. someone"), "alpha");
    assert_eq!(strip_decorations("alpha (part 2)"), "alpha (part 2)");
    assert_eq!(strip_decorations("credit"), "credit");
}

#[test]
fn base_text_normalizes_case_and_whitespace() {
    assert_eq!(base_text("  Album   XYZ  "), "album xyz");
}

#[test]
fn base_text_strips_decoration_tags() {
    assert_eq!(base_text("Album XYZ (Deluxe)"), "album xyz");
    assert_eq!(base_text("Album XYZ [Remastered 2011]"), "album xyz");
    assert_eq!(base_text("Album XYZ - Live"), "album xyz");
}

#[test]
fn base_text_keeps_meaningful_parentheses() {
    assert_eq!(base_text("Album XYZ (Part 2)"), "album xyz (part 2)");
}
