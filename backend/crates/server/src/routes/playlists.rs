//! Playlist CRUD handlers and the `PlaylistsApi` OpenAPI spec struct.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use uuid::Uuid;

use api_types::{
    common::ErrorResponse,
    playlists::{
        AddPerformanceRequest, CreatePlaylistRequest, PlaylistKind, PlaylistResponse,
        UpdatePlaylistRequest,
    },
};
use db::{
    error::DbError,
    models::playlist::{NewPlaylist, UpdatePlaylist},
    queries,
};

use crate::{auth::middleware::AuthUser, capabilities, error::ApiError, state::AppState};

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        list_playlists,
        get_playlist,
        create_playlist,
        update_playlist,
        delete_playlist,
        list_playlist_performances,
        add_playlist_performance,
        remove_playlist_performance,
    ),
    components(schemas(
        PlaylistResponse,
        PlaylistKind,
        CreatePlaylistRequest,
        UpdatePlaylistRequest,
        AddPerformanceRequest,
        ErrorResponse,
    ))
)]
pub(crate) struct PlaylistsApi;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_playlists).post(create_playlist))
        .route(
            "/{id}",
            get(get_playlist)
                .put(update_playlist)
                .delete(delete_playlist),
        )
        .route(
            "/{id}/performances",
            get(list_playlist_performances).post(add_playlist_performance),
        )
        .route(
            "/{id}/performances/{perf_id}",
            axum::routing::delete(remove_playlist_performance),
        )
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
    path = "/api/playlists",
    responses(
        (status = 200, description = "Returns public playlists, or all playlists for users with sufficient permissions.", body = Vec<PlaylistResponse>),
    ),
    tag = "playlists"
)]
pub(crate) async fn list_playlists(
    State(state): State<AppState>,
    auth: Option<AuthUser>,
) -> Result<Json<Vec<PlaylistResponse>>, ApiError> {
    let can_view_private = auth.is_some_and(|u| {
        u.capabilities
            .iter()
            .any(|c| c == capabilities::VIEW_PRIVATE_PLAYLISTS)
    });
    let playlists = if can_view_private {
        queries::playlists::list(&state.pool).await?
    } else {
        queries::playlists::list_public(&state.pool).await?
    };
    let items = playlists
        .into_iter()
        .map(to_response)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(items))
}

#[utoipa::path(
    get,
    path = "/api/playlists/{id}",
    params(("id" = Uuid, Path, description = "Playlist ID")),
    responses(
        (status = 200, description = "Playlist detail", body = PlaylistResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "playlists"
)]
pub(crate) async fn get_playlist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PlaylistResponse>, ApiError> {
    let playlist = queries::playlists::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(to_response(playlist)?))
}

#[utoipa::path(
    post,
    path = "/api/playlists",
    request_body = CreatePlaylistRequest,
    responses(
        (status = 201, description = "Created playlist", body = PlaylistResponse),
        (status = 401, description = "Not authenticated", body = ErrorResponse),
    ),
    tag = "playlists"
)]
pub(crate) async fn create_playlist(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreatePlaylistRequest>,
) -> Result<(StatusCode, Json<PlaylistResponse>), ApiError> {
    let mut conn = state.pool.acquire().await.map_err(DbError::Sqlx)?;
    let playlist = queries::playlists::create(
        &mut conn,
        &NewPlaylist {
            title: req.title,
            description: req.description,
            kind: req.kind.as_str().to_owned(),
            is_public: req.is_public,
            created_by: Some(auth.user_id),
        },
    )
    .await?;
    Ok((StatusCode::CREATED, Json(to_response(playlist)?)))
}

#[utoipa::path(
    put,
    path = "/api/playlists/{id}",
    params(("id" = Uuid, Path, description = "Playlist ID")),
    request_body = UpdatePlaylistRequest,
    responses(
        (status = 200, description = "Updated playlist", body = PlaylistResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "playlists"
)]
pub(crate) async fn update_playlist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePlaylistRequest>,
) -> Result<Json<PlaylistResponse>, ApiError> {
    let mut conn = state.pool.acquire().await.map_err(DbError::Sqlx)?;
    let playlist = queries::playlists::update(
        &mut conn,
        id,
        &UpdatePlaylist {
            title: req.title,
            description: req.description,
            kind: req.kind.as_str().to_owned(),
            is_public: req.is_public,
        },
    )
    .await?
    .ok_or(ApiError::NotFound)?;
    Ok(Json(to_response(playlist)?))
}

#[utoipa::path(
    delete,
    path = "/api/playlists/{id}",
    params(("id" = Uuid, Path, description = "Playlist ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "playlists"
)]
pub(crate) async fn delete_playlist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let found = queries::playlists::delete(&state.pool, id).await?;
    if found {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

#[utoipa::path(
    get,
    path = "/api/playlists/{id}/performances",
    params(("id" = Uuid, Path, description = "Playlist ID")),
    responses(
        (status = 200, description = "Ordered performance IDs in this playlist", body = Vec<Uuid>),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "playlists"
)]
pub(crate) async fn list_playlist_performances(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Uuid>>, ApiError> {
    queries::playlists::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let ids = queries::playlists::get_performance_ids(&state.pool, id).await?;
    Ok(Json(ids))
}

#[utoipa::path(
    post,
    path = "/api/playlists/{id}/performances",
    params(("id" = Uuid, Path, description = "Playlist ID")),
    request_body = AddPerformanceRequest,
    responses(
        (status = 204, description = "Performance added"),
        (status = 404, description = "Playlist not found", body = ErrorResponse),
    ),
    tag = "playlists"
)]
pub(crate) async fn add_playlist_performance(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<AddPerformanceRequest>,
) -> Result<StatusCode, ApiError> {
    queries::playlists::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    queries::playlists::add_performance(&state.pool, id, req.performance_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/playlists/{id}/performances/{perf_id}",
    params(
        ("id" = Uuid, Path, description = "Playlist ID"),
        ("perf_id" = Uuid, Path, description = "Performance ID"),
    ),
    responses(
        (status = 204, description = "Performance removed"),
        (status = 404, description = "Playlist not found", body = ErrorResponse),
    ),
    tag = "playlists"
)]
pub(crate) async fn remove_playlist_performance(
    State(state): State<AppState>,
    Path((id, perf_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    queries::playlists::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    queries::playlists::remove_performance(&state.pool, id, perf_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
