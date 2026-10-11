//! Errors that prevent a library export from being parsed at all

use thiserror::Error;

/// File-level problems that make an export unusable
///
/// Item-level problems such as tracks without required metadata do not appear here; they are counted in [`crate::parser::ParseReport`]
#[derive(Debug, Error)]
pub enum ParseError {
    /// The input is not a valid XML property list
    #[error("input is not a valid XML property list: {0}")]
    InvalidPlist(#[from] plist::Error),
    /// A required top-level section is missing from the export
    #[error("library export is missing the `{0}` section")]
    MissingSection(&'static str),
}
