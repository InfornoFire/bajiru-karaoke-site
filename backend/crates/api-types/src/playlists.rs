//! Playlist resource types.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Valid kind values for a playlist.
///
/// `Favorites` playlists are created automatically on user registration and
/// cannot be created or deleted through normal playlist endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlaylistKind {
    User,
    Official,
    Favorites,
}

impl PlaylistKind {
    /// Returns the string stored in the database for this kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Official => "official",
            Self::Favorites => "favorites",
        }
    }
}
