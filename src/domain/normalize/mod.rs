//! Text normalization used when comparing metadata across tracks

/// Trims, collapses internal whitespace runs, and lowercases the input
///
/// The result is used as an interning key and as the comparison key for analyses, so spelling variants that differ only in case or whitespace compare equal
#[must_use]
pub fn normalize_text(input: &str) -> String {
    let mut normalized = String::with_capacity(input.len());
    for word in input.split_whitespace() {
        if !normalized.is_empty() {
            normalized.push(' ');
        }
        normalized.extend(word.chars().flat_map(char::to_lowercase));
    }
    normalized
}

#[cfg(test)]
mod tests;
