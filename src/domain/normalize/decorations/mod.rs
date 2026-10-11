//! Decoration stripping for titles that carry edition tags

use super::normalize_text;

/// Keywords that mark a bracketed or dashed segment as a decoration
const DECORATION_KEYWORDS: &[&str] = &[
    "acoustic",
    "anniversary",
    "bonus",
    "deluxe",
    "demo",
    "edit",
    "edition",
    "explicit",
    "extended",
    "feat",
    "ft",
    "instrumental",
    "live",
    "mix",
    "mono",
    "radio",
    "remaster",
    "remix",
    "reprise",
    "session",
    "single",
    "stereo",
    "take",
    "version",
];

/// Lowercases, collapses whitespace, and drops decoration segments such as
/// `(Remastered 2011)`, `[Deluxe Edition]`, `- Live`, or `feat. X`
///
/// The result is the comparison key used by analyses that match titles across editions of the same release
#[must_use]
pub fn base_text(input: &str) -> String {
    normalize_text(&strip_decorations(&normalize_text(input)))
}

fn strip_decorations(input: &str) -> String {
    let mut stripped = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(open) = rest.find(['(', '[']) {
        let (before, from_open) = rest.split_at(open);
        stripped.push_str(before);
        let close = if from_open.starts_with('(') { ')' } else { ']' };
        let Some(end) = from_open.find(close) else {
            stripped.push_str(from_open);
            rest = "";
            break;
        };
        if !contains_decoration(&from_open[1..end]) {
            stripped.push_str(&from_open[..=end]);
        }
        rest = &from_open[end + 1..];
    }
    stripped.push_str(rest);

    let kept: Vec<&str> = stripped
        .split(" - ")
        .enumerate()
        .filter(|(index, segment)| *index == 0 || !contains_decoration(segment))
        .map(|(_, segment)| segment)
        .collect();
    let dashed = kept.join(" - ");

    let stripped = match dashed.find(" feat") {
        Some(position) => &dashed[..position],
        None => &dashed,
    };
    stripped.trim().to_owned()
}

fn contains_decoration(segment: &str) -> bool {
    segment
        .split(|character: char| !character.is_alphanumeric())
        .any(|token| {
            DECORATION_KEYWORDS
                .iter()
                .any(|keyword| token.starts_with(keyword))
        })
}

#[cfg(test)]
mod tests;
