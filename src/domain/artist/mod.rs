use crate::domain::ids::ArtistId;

/// A performing artist, interned during parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artist {
    /// Stable id assigned during parsing
    pub id: ArtistId,
    /// Display name taken from the first track that mentioned the artist
    pub name: String,
}
