//! Parser for the official Apple Music XML library export

mod convert;
mod raw;

use crate::parser::ParseError;
use crate::parser::ParseOutput;

use raw::RawLibrary;

/// Parses an Apple Music XML export
///
/// # Errors
///
/// Returns [`ParseError`] when the bytes are not a valid XML property list or either the `Tracks` or `Playlists` section is missing
pub(crate) fn parse(input: &[u8]) -> Result<ParseOutput, ParseError> {
    let raw: RawLibrary = plist::from_bytes(input)?;
    let tracks = raw.tracks.ok_or(ParseError::MissingSection("Tracks"))?;
    let playlists = raw
        .playlists
        .ok_or(ParseError::MissingSection("Playlists"))?;
    let (library, report) = convert::convert(tracks, playlists);
    Ok(ParseOutput { library, report })
}

#[cfg(test)]
mod tests;
