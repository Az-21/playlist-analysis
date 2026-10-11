//! Parsing of library exports into the domain model

mod apple_music;
mod error;

use crate::domain::Library;
pub use error::ParseError;

/// Music streaming services supported as export sources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Service {
    /// Apple Music official library export
    #[default]
    AppleMusic,
}

/// Artifact formats supported for a service
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LibraryFormat {
    /// XML property list
    #[default]
    Xml,
}

/// A concrete source to parse from
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceKind {
    /// Streaming service the artifact came from
    pub service: Service,
    pub format: LibraryFormat,
}

/// Counts of recoverable problems encountered while parsing
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ParseReport {
    /// Tracks whose dict key was malformed or that lacked required metadata
    pub skipped_tracks: usize,
    /// Playlist items that referenced an unknown or malformed track id
    pub skipped_playlist_entries: usize,
    /// Playlists that lacked a name or id
    pub skipped_playlists: usize,
}

/// A parsed library plus the problems encountered along the way
#[derive(Debug)]
pub struct ParseOutput {
    pub library: Library,
    /// Recoverable problems found during parsing
    pub report: ParseReport,
}

/// Detects the source kind from the first bytes of an export
#[must_use]
pub fn detect(input: &[u8]) -> Option<SourceKind> {
    let head = &input[..input.len().min(1_024)];
    let text = String::from_utf8_lossy(head);
    (text.contains("<?xml") && text.contains("<plist")).then_some(SourceKind {
        service: Service::AppleMusic,
        format: LibraryFormat::Xml,
    })
}

/// Parses an export with the given source kind
///
/// # Errors
///
/// Returns [`ParseError`] when the input is not a valid artifact of the requested source, or when a required top-level section is missing
pub fn parse(source: SourceKind, input: &[u8]) -> Result<ParseOutput, ParseError> {
    match (source.service, source.format) {
        (Service::AppleMusic, LibraryFormat::Xml) => apple_music::xml::parse(input),
    }
}

#[cfg(test)]
mod tests;
