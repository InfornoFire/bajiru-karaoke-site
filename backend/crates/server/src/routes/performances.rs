//! Performance CRUD handlers, media upload/delete, and the `PerformancesApi` OpenAPI spec struct.

pub(crate) mod lyrics;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use tracing::error;

use api_types::{
    common::{ArtistInfo, ErrorResponse, MediaInfo},
    lyrics::{LyricsResponse, UpdateLyricsRequest},
    performances::{
        CreatePerformanceRequest, PerformanceResponse, PerformanceSummary, UpdatePerformanceRequest,
    },
    songs::SongSummary,
};

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        list_performances,
        get_performance,
        create_performance,
        update_performance,
        delete_performance,
        upload_audio,
        delete_audio,
        upload_video,
        delete_video,
        lyrics::get_lyrics,
        lyrics::put_lyrics,
        lyrics::delete_lyrics,
    ),
    components(schemas(
        PerformanceSummary,
        PerformanceResponse,
        CreatePerformanceRequest,
        UpdatePerformanceRequest,
        SongSummary,
        ArtistInfo,
        MediaInfo,
        FileUpload,
        LyricsResponse,
        UpdateLyricsRequest,
        ErrorResponse,
    ))
)]
pub(crate) struct PerformancesApi;
use db::{
    MySqlPool,
    models::{
        NewLyrics, NewPerformance, NewPerformanceAudio, NewPerformanceVideo, UpdatePerformance,
    },
    queries,
};

use crate::{error::ApiError, media, state::AppState};

/// Placeholder schema for multipart file upload bodies.
#[derive(utoipa::ToSchema)]
#[allow(dead_code)]
pub(crate) struct FileUpload {
    #[schema(format = Binary)]
    pub file: Vec<u8>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_performances).post(create_performance))
        .route(
            "/{id}",
            get(get_performance)
                .put(update_performance)
                .delete(delete_performance),
        )
        .route(
            "/{id}/audio",
            post(upload_audio).layer(DefaultBodyLimit::max(500 * 1024 * 1024)),
        )
        .route("/{id}/audio/{audio_id}", delete(delete_audio))
        .route(
            "/{id}/video",
            post(upload_video).layer(DefaultBodyLimit::max(500 * 1024 * 1024)),
        )
        .route("/{id}/video/{video_id}", delete(delete_video))
        .route(
            "/{id}/lyrics",
            get(lyrics::get_lyrics)
                .put(lyrics::put_lyrics)
                .delete(lyrics::delete_lyrics),
        )
}

/// Loads all related entities for a performance row into a full [`PerformanceResponse`].
async fn hydrate(
    pool: &MySqlPool,
    perf: db::models::Performance,
) -> Result<PerformanceResponse, ApiError> {
    let songs = queries::performances::get_songs(pool, perf.id)
        .await?
        .into_iter()
        .map(|s| SongSummary {
            id: s.id,
            title: s.title,
            date_added: s.date_added,
            artists: vec![],
        })
        .collect();

    let singers = queries::performances::get_singers(pool, perf.id)
        .await?
        .into_iter()
        .map(|a| ArtistInfo {
            id: a.id,
            name: a.name,
            description: a.description,
        })
        .collect();

    let audio = queries::performance_audios::list_for_performance(pool, perf.id)
        .await?
        .into_iter()
        .map(|a| MediaInfo {
            id: a.id,
            public_url: a.public_url,
        })
        .collect();

    let video = queries::performance_videos::list_for_performance(pool, perf.id)
        .await?
        .into_iter()
        .map(|v| MediaInfo {
            id: v.id,
            public_url: v.public_url,
        })
        .collect();

    Ok(PerformanceResponse {
        id: perf.id,
        title: perf.title,
        play_count: perf.play_count,
        duration: perf.duration,
        performance_date: perf.performance_date,
        songs,
        singers,
        audio,
        video,
    })
}

#[utoipa::path(
    get,
    path = "/api/performances",
    responses(
        (status = 200, description = "List of performances (summarized)", body = Vec<PerformanceSummary>),
    ),
    tag = "performances"
)]
pub(crate) async fn list_performances(
    State(state): State<AppState>,
) -> Result<Json<Vec<PerformanceSummary>>, ApiError> {
    let perfs = queries::performances::list(&state.pool).await?;
    let perf_ids: Vec<u32> = perfs.iter().map(|p| p.id).collect();
    let mut singers_by_perf =
        queries::performances::get_singers_batch(&state.pool, &perf_ids).await?;

    let summaries = perfs
        .into_iter()
        .map(|p| {
            let singers = singers_by_perf
                .remove(&p.id)
                .unwrap_or_default()
                .into_iter()
                .map(|a| ArtistInfo {
                    id: a.id,
                    name: a.name,
                    description: a.description,
                })
                .collect();
            PerformanceSummary {
                id: p.id,
                title: p.title,
                play_count: p.play_count,
                duration: p.duration,
                performance_date: p.performance_date,
                singers,
            }
        })
        .collect();
    Ok(Json(summaries))
}

#[utoipa::path(
    get,
    path = "/api/performances/{id}",
    params(("id" = u32, Path, description = "Performance ID")),
    responses(
        (status = 200, description = "Performance detail", body = PerformanceResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "performances"
)]
pub(crate) async fn get_performance(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<PerformanceResponse>, ApiError> {
    let perf = queries::performances::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(hydrate(&state.pool, perf).await?))
}

#[utoipa::path(
    post,
    path = "/api/performances",
    request_body = CreatePerformanceRequest,
    responses(
        (status = 201, description = "Created performance", body = PerformanceResponse),
    ),
    tag = "performances"
)]
pub(crate) async fn create_performance(
    State(state): State<AppState>,
    Json(req): Json<CreatePerformanceRequest>,
) -> Result<(StatusCode, Json<PerformanceResponse>), ApiError> {
    let lyrics_id = match req.lyrics {
        Some(content) => {
            let l = queries::lyrics::create(&state.pool, &NewLyrics { content }).await?;
            Some(l.id)
        }
        None => None,
    };

    let perf = queries::performances::create(
        &state.pool,
        &NewPerformance {
            created_by: None,
            title: req.title,
            lyrics_id,
            duration: req.duration,
            performance_date: req.performance_date,
        },
    )
    .await?;

    queries::performances::set_songs(&state.pool, perf.id, &req.song_ids).await?;
    queries::performances::set_singers(&state.pool, perf.id, &req.singer_ids).await?;

    Ok((StatusCode::CREATED, Json(hydrate(&state.pool, perf).await?)))
}

#[utoipa::path(
    put,
    path = "/api/performances/{id}",
    params(("id" = u32, Path, description = "Performance ID")),
    request_body = UpdatePerformanceRequest,
    responses(
        (status = 200, description = "Updated performance", body = PerformanceResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "performances"
)]
pub(crate) async fn update_performance(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(req): Json<UpdatePerformanceRequest>,
) -> Result<Json<PerformanceResponse>, ApiError> {
    let perf = queries::performances::update(
        &state.pool,
        id,
        &UpdatePerformance {
            title: req.title,
            duration: req.duration,
            performance_date: req.performance_date,
        },
    )
    .await?
    .ok_or(ApiError::NotFound)?;

    queries::performances::set_songs(&state.pool, id, &req.song_ids).await?;
    queries::performances::set_singers(&state.pool, id, &req.singer_ids).await?;

    Ok(Json(hydrate(&state.pool, perf).await?))
}

#[utoipa::path(
    delete,
    path = "/api/performances/{id}",
    params(("id" = u32, Path, description = "Performance ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "performances"
)]
pub(crate) async fn delete_performance(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<StatusCode, ApiError> {
    let found = queries::performances::delete(&state.pool, id).await?;
    if found {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

/// Reads the `file` field from a multipart body and returns its bytes, content type, and filename.
async fn read_file_field(
    multipart: &mut Multipart,
) -> Result<(Vec<u8>, String, Option<String>), ApiError> {
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("multipart field error: {e:?}");
        ApiError::BadRequest(e.to_string())
    })? {
        if field.name() == Some("file") {
            let content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            let filename = field.file_name().map(str::to_string);
            let data = field
                .bytes()
                .await
                .map_err(|e| {
                    error!("multipart read error: {e:?}");
                    ApiError::BadRequest(e.to_string())
                })?
                .to_vec();
            return Ok((data, content_type, filename));
        }
    }
    Err(ApiError::BadRequest("missing 'file' field".into()))
}

#[utoipa::path(
    post,
    path = "/api/performances/{id}/audio",
    params(("id" = u32, Path, description = "Performance ID")),
    request_body(content = FileUpload, content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "Audio uploaded", body = MediaInfo),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 404, description = "Performance not found", body = ErrorResponse),
    ),
    tag = "performances"
)]
pub(crate) async fn upload_audio(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<MediaInfo>), ApiError> {
    queries::performances::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let (data, content_type, filename) = read_file_field(&mut multipart).await?;
    let ext = media::resolve_ext(media::MediaKind::Audio, &content_type, filename.as_deref())?;
    let saved = state.store.save("audio", ext, &data).await?;

    let audio = queries::performance_audios::create(
        &state.pool,
        &NewPerformanceAudio {
            performance_id: id,
            public_url: saved.public_url,
            internal_path: Some(saved.internal_path),
        },
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(MediaInfo {
            id: audio.id,
            public_url: audio.public_url,
        }),
    ))
}

#[utoipa::path(
    delete,
    path = "/api/performances/{id}/audio/{audio_id}",
    params(
        ("id" = u32, Path, description = "Performance ID"),
        ("audio_id" = u32, Path, description = "Audio record ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "performances"
)]
pub(crate) async fn delete_audio(
    State(state): State<AppState>,
    Path((id, audio_id)): Path<(u32, u32)>,
) -> Result<StatusCode, ApiError> {
    let audio = queries::performance_audios::get_by_id(&state.pool, audio_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if audio.performance_id != id {
        return Err(ApiError::NotFound);
    }
    queries::performance_audios::delete(&state.pool, audio_id).await?;
    if let Some(path) = &audio.internal_path
        && let Err(e) = state.store.delete(path).await
    {
        error!("failed to delete audio file {path}: {e}");
    }
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/performances/{id}/video",
    params(("id" = u32, Path, description = "Performance ID")),
    request_body(content = FileUpload, content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "Video uploaded", body = MediaInfo),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 404, description = "Performance not found", body = ErrorResponse),
    ),
    tag = "performances"
)]
pub(crate) async fn upload_video(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<MediaInfo>), ApiError> {
    queries::performances::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let (data, content_type, filename) = read_file_field(&mut multipart).await?;
    let ext = media::resolve_ext(media::MediaKind::Video, &content_type, filename.as_deref())?;
    let saved = state.store.save("video", ext, &data).await?;

    let video = queries::performance_videos::create(
        &state.pool,
        &NewPerformanceVideo {
            performance_id: id,
            public_url: saved.public_url,
            internal_path: Some(saved.internal_path),
        },
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(MediaInfo {
            id: video.id,
            public_url: video.public_url,
        }),
    ))
}

#[utoipa::path(
    delete,
    path = "/api/performances/{id}/video/{video_id}",
    params(
        ("id" = u32, Path, description = "Performance ID"),
        ("video_id" = u32, Path, description = "Video record ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "performances"
)]
pub(crate) async fn delete_video(
    State(state): State<AppState>,
    Path((id, video_id)): Path<(u32, u32)>,
) -> Result<StatusCode, ApiError> {
    let video = queries::performance_videos::get_by_id(&state.pool, video_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if video.performance_id != id {
        return Err(ApiError::NotFound);
    }
    queries::performance_videos::delete(&state.pool, video_id).await?;
    if let Some(path) = &video.internal_path
        && let Err(e) = state.store.delete(path).await
    {
        error!("failed to delete video file {path}: {e}");
    }
    Ok(StatusCode::NO_CONTENT)
}
