//! Capability title constants used to guard routes.
//!
//! Each constant matches a row in the `capabilities` table. Handlers compare
//! against [`AuthUser::capabilities`] which is fetched fresh on every request.

pub const VIEW_PRIVATE_PLAYLISTS: &str = "playlists:view_private";
