//! Song CRUD handlers and the `SongsApi` OpenAPI spec struct.

pub(crate) mod lyrics;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};

use api_types::{
    common::{ArtistInfo, ErrorResponse, ImageInfo, TagInfo},
    lyrics::{LyricsResponse, UpdateLyricsRequest},
    songs::{CreateSongRequest, SongResponse, SongSummary, UpdateSongRequest},
};

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        list_songs,
        get_song,
        create_song,
        update_song,
        delete_song,
        lyrics::get_lyrics,
        lyrics::put_lyrics,
        lyrics::delete_lyrics,
    ),
    components(schemas(
        SongSummary,
        SongResponse,
        CreateSongRequest,
        UpdateSongRequest,
        LyricsResponse,
        UpdateLyricsRequest,
        ArtistInfo,
        TagInfo,
        ImageInfo,
        ErrorResponse,
    ))
)]
pub(crate) struct SongsApi;
use db::{
    MySqlPool,
    models::{NewLyrics, NewSong, UpdateSong},
    queries,
};

use crate::{error::ApiError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_songs).post(create_song))
        .route("/{id}", get(get_song).put(update_song).delete(delete_song))
        .route(
            "/{id}/lyrics",
            get(lyrics::get_lyrics)
                .put(lyrics::put_lyrics)
                .delete(lyrics::delete_lyrics),
        )
}

/// Loads all related entities for a song row into a full [`SongResponse`].
async fn hydrate(pool: &MySqlPool, song: db::models::Song) -> Result<SongResponse, ApiError> {
    let (artists, tags, images) = tokio::try_join!(
        queries::songs::get_original_artists(pool, song.id),
        queries::songs::get_tags(pool, song.id),
        queries::songs::get_images(pool, song.id),
    )?;

    let artists = artists
        .into_iter()
        .map(|a| ArtistInfo {
            id: a.id,
            name: a.name,
            description: a.description,
        })
        .collect();

    let tags = tags
        .into_iter()
        .map(|t| TagInfo {
            id: t.id,
            name: t.name,
            kind: t.kind,
        })
        .collect();

    let images = images
        .into_iter()
        .map(|i| ImageInfo {
            id: i.id,
            public_url: i.public_url,
            credits: i.credits,
        })
        .collect();

    Ok(SongResponse {
        id: song.id,
        title: song.title,
        date_added: song.date_added,
        artists,
        tags,
        images,
    })
}

#[utoipa::path(
    get,
    path = "/api/songs",
    responses(
        (status = 200, description = "List of songs (summarized)", body = Vec<SongSummary>),
    ),
    tag = "songs"
)]
pub(crate) async fn list_songs(
    State(state): State<AppState>,
) -> Result<Json<Vec<SongSummary>>, ApiError> {
    let songs = queries::songs::list(&state.pool).await?;
    let song_ids: Vec<u32> = songs.iter().map(|s| s.id).collect();
    let mut artists_by_song =
        queries::songs::get_original_artists_batch(&state.pool, &song_ids).await?;

    let summaries = songs
        .into_iter()
        .map(|s| {
            let artists = artists_by_song
                .remove(&s.id)
                .unwrap_or_default()
                .into_iter()
                .map(|a| ArtistInfo {
                    id: a.id,
                    name: a.name,
                    description: a.description,
                })
                .collect();
            SongSummary {
                id: s.id,
                title: s.title,
                date_added: s.date_added,
                artists,
            }
        })
        .collect();
    Ok(Json(summaries))
}

#[utoipa::path(
    get,
    path = "/api/songs/{id}",
    params(("id" = u32, Path, description = "Song ID")),
    responses(
        (status = 200, description = "Song detail", body = SongResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "songs"
)]
pub(crate) async fn get_song(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<SongResponse>, ApiError> {
    let song = queries::songs::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(hydrate(&state.pool, song).await?))
}

#[utoipa::path(
    post,
    path = "/api/songs",
    request_body = CreateSongRequest,
    responses(
        (status = 201, description = "Created song", body = SongResponse),
    ),
    tag = "songs"
)]
pub(crate) async fn create_song(
    State(state): State<AppState>,
    Json(req): Json<CreateSongRequest>,
) -> Result<(StatusCode, Json<SongResponse>), ApiError> {
    let lyrics_id = match req.lyrics {
        Some(content) => {
            let l = queries::lyrics::create(&state.pool, &NewLyrics { content }).await?;
            Some(l.id)
        }
        None => None,
    };

    let song = queries::songs::create(
        &state.pool,
        &NewSong {
            title: req.title,
            created_by: None,
            lyrics_id,
        },
    )
    .await?;

    queries::songs::set_original_artists(&state.pool, song.id, &req.artist_ids).await?;
    queries::songs::set_tags(&state.pool, song.id, &req.tag_ids).await?;
    queries::songs::set_images(&state.pool, song.id, &req.image_ids).await?;

    Ok((StatusCode::CREATED, Json(hydrate(&state.pool, song).await?)))
}

#[utoipa::path(
    put,
    path = "/api/songs/{id}",
    params(("id" = u32, Path, description = "Song ID")),
    request_body = UpdateSongRequest,
    responses(
        (status = 200, description = "Updated song", body = SongResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "songs"
)]
pub(crate) async fn update_song(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(req): Json<UpdateSongRequest>,
) -> Result<Json<SongResponse>, ApiError> {
    let song = queries::songs::update(&state.pool, id, &UpdateSong { title: req.title })
        .await?
        .ok_or(ApiError::NotFound)?;

    queries::songs::set_original_artists(&state.pool, id, &req.artist_ids).await?;
    queries::songs::set_tags(&state.pool, id, &req.tag_ids).await?;
    queries::songs::set_images(&state.pool, id, &req.image_ids).await?;

    Ok(Json(hydrate(&state.pool, song).await?))
}

#[utoipa::path(
    delete,
    path = "/api/songs/{id}",
    params(("id" = u32, Path, description = "Song ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "songs"
)]
pub(crate) async fn delete_song(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<StatusCode, ApiError> {
    let found = queries::songs::delete(&state.pool, id).await?;
    if found {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}
