//! Service-neutral domain model shared by the parser, analysis, and export layers

mod album;
mod artist;
mod ids;
mod library;
mod normalize;
mod playlist;
mod song;

pub use album::Album;
pub use artist::Artist;
pub use ids::{AlbumId, ArtistId, PlaylistId, SongId};
pub use library::{Library, format_duration};
pub use normalize::normalize_text;
pub use playlist::Playlist;
pub use song::Song;
