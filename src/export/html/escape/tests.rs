use super::*;

#[test]
fn escapes_every_special_character() {
    assert_eq!(escape_html("&"), "&amp;");
    assert_eq!(escape_html("<"), "&lt;");
    assert_eq!(escape_html(">"), "&gt;");
    assert_eq!(escape_html("\""), "&quot;");
    assert_eq!(escape_html("'"), "&#39;");
}

#[test]
fn escapes_mixed_markup() {
    assert_eq!(
        escape_html(r#"<script>alert("hi")</script>"#),
        "&lt;script&gt;alert(&quot;hi&quot;)&lt;/script&gt;"
    );
}

#[test]
fn leaves_plain_text_and_non_ascii_untouched() {
    assert_eq!(escape_html("Alpha Beta \u{c9}"), "Alpha Beta \u{c9}");
    assert_eq!(escape_html(""), "");
}
