use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use api_types::{
    common::ErrorResponse,
    lyrics::{LyricsResponse, UpdateLyricsRequest},
};
use db::{models::NewLyrics, queries};

use crate::{error::ApiError, state::AppState};

#[utoipa::path(
    get,
    path = "/api/performances/{id}/lyrics",
    params(("id" = u32, Path, description = "Performance ID")),
    responses(
        (status = 200, description = "Lyrics for this performance. Returns performance-specific \
                                      lyrics if set, otherwise falls back to the linked song's \
                                      lyrics.", body = LyricsResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "performances"
)]
pub(crate) async fn get_lyrics(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<LyricsResponse>, ApiError> {
    let perf = queries::performances::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if let Some(lyrics_id) = perf.lyrics_id {
        let lyrics = queries::lyrics::get_by_id(&state.pool, lyrics_id)
            .await?
            .ok_or(ApiError::NotFound)?;
        return Ok(Json(LyricsResponse {
            content: lyrics.content,
        }));
    }

    let content = queries::performances::get_fallback_song_lyrics(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(LyricsResponse { content }))
}

#[utoipa::path(
    put,
    path = "/api/performances/{id}/lyrics",
    params(("id" = u32, Path, description = "Performance ID")),
    request_body = UpdateLyricsRequest,
    responses(
        (status = 204, description = "Lyrics saved"),
        (status = 404, description = "Performance not found", body = ErrorResponse),
    ),
    tag = "performances"
)]
pub(crate) async fn put_lyrics(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(req): Json<UpdateLyricsRequest>,
) -> Result<StatusCode, ApiError> {
    let perf = queries::performances::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    match perf.lyrics_id {
        Some(existing_id) => {
            queries::lyrics::update(&state.pool, existing_id, &req.content).await?;
        }
        None => {
            let lyrics = queries::lyrics::create(
                &state.pool,
                &NewLyrics {
                    content: req.content,
                },
            )
            .await?;
            queries::performances::update_lyrics_id(&state.pool, id, Some(lyrics.id)).await?;
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/performances/{id}/lyrics",
    params(("id" = u32, Path, description = "Performance ID")),
    responses(
        (status = 204, description = "Performance lyrics override removed"),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "performances"
)]
pub(crate) async fn delete_lyrics(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<StatusCode, ApiError> {
    let perf = queries::performances::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let Some(lyrics_id) = perf.lyrics_id else {
        return Err(ApiError::NotFound);
    };

    queries::performances::update_lyrics_id(&state.pool, id, None).await?;

    let refs = queries::lyrics::reference_count(&state.pool, lyrics_id).await?;
    if refs == 0 {
        queries::lyrics::delete(&state.pool, lyrics_id).await?;
    }

    Ok(StatusCode::NO_CONTENT)
}
