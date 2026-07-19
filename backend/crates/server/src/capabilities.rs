//! Capability title constants and authorization mappings.
//!
//! Each constant matches a row in the `capabilities` table. Handlers compare
//! against [`AuthUser::capabilities`] which is fetched fresh on every request.

use api_types::playlists::PlaylistKind;

pub const VIEW_PRIVATE_PLAYLISTS: &str = "playlists:view_private";
pub const CREATE_OFFICIAL_PLAYLIST: &str = "playlists:create_official";
pub const CREATE_FAVORITES_PLAYLIST: &str = "playlists:create_favorites";

/// Returns the capability required to create a playlist of the given kind, or
/// `None` if any authenticated user may create it.
pub fn required_create_playlist_capability(kind: &PlaylistKind) -> Option<&'static str> {
    match kind {
        PlaylistKind::User => None,
        PlaylistKind::Official => Some(CREATE_OFFICIAL_PLAYLIST),
        PlaylistKind::Favorites => Some(CREATE_FAVORITES_PLAYLIST),
    }
}
