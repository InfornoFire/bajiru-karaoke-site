//! User scoped playlist handlers and the `UsersApi` OpenAPI spec struct.

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use uuid::Uuid;

use api_types::{
    common::ErrorResponse,
    playlists::{PlaylistKind, PlaylistResponse},
};
use db::queries;

use crate::{auth::middleware::AuthUser, capabilities, error::ApiError, state::AppState};

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(list_user_playlists, get_user_favorites),
    components(schemas(PlaylistResponse, PlaylistKind, ErrorResponse))
)]
pub(crate) struct UsersApi;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{id}/playlists", get(list_user_playlists))
        .route("/{id}/favorites", get(get_user_favorites))
}

fn can_view_private(auth: &Option<AuthUser>, user_id: Uuid) -> bool {
    auth.as_ref().is_some_and(|u| {
        u.user_id == user_id
            || u.capabilities
                .iter()
                .any(|c| c == capabilities::VIEW_PRIVATE_PLAYLISTS)
    })
}

fn to_response(p: db::models::Playlist) -> Result<PlaylistResponse, ApiError> {
    let kind = match p.kind.as_str() {
        "user" => PlaylistKind::User,
        "official" => PlaylistKind::Official,
        "favorites" => PlaylistKind::Favorites,
        other => {
            return Err(ApiError::Internal(format!(
                "unknown playlist kind in database: {other}"
            )));
        }
    };
    Ok(PlaylistResponse {
        id: p.id,
        title: p.title,
        description: p.description,
        kind,
        is_public: p.is_public,
        created_by: p.created_by,
    })
}

#[utoipa::path(
    get,
    path = "/api/users/{id}/playlists",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "Playlists for this user. Returns all playlists when viewing your own profile or with sufficient permissions, otherwise public only.", body = Vec<PlaylistResponse>),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    tag = "users"
)]
pub(crate) async fn list_user_playlists(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    auth: Option<AuthUser>,
) -> Result<Json<Vec<PlaylistResponse>>, ApiError> {
    queries::users::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let playlists = if can_view_private(&auth, id) {
        queries::playlists::list_by_user(&state.pool, id).await?
    } else {
        queries::playlists::list_public_by_user(&state.pool, id).await?
    };

    let items = playlists
        .into_iter()
        .map(to_response)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(items))
}

#[utoipa::path(
    get,
    path = "/api/users/{id}/favorites",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "Ordered performance IDs in this user's favorites playlist.", body = Vec<Uuid>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    tag = "users"
)]
pub(crate) async fn get_user_favorites(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    auth: Option<AuthUser>,
) -> Result<Json<Vec<Uuid>>, ApiError> {
    if !can_view_private(&auth, id) {
        return Err(ApiError::Forbidden);
    }

    queries::users::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let playlist = queries::playlists::get_favorites_by_user(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let ids = queries::playlists::get_performance_ids(&state.pool, playlist.id).await?;
    Ok(Json(ids))
}
