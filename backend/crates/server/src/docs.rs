use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use api_types::{
    common::{ArtistInfo, ErrorResponse, ImageInfo, MediaInfo, TagInfo},
    lyrics::{LyricsResponse, UpdateLyricsRequest},
    performances::{
        CreatePerformanceRequest, PerformanceResponse, PerformanceSummary, UpdatePerformanceRequest,
    },
    songs::{CreateSongRequest, SongResponse, SongSummary, UpdateSongRequest},
};

use crate::{routes::performances::FileUpload, state::AppState};

/// Public API Docs
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::songs::list_songs,
        crate::routes::songs::get_song,
        crate::routes::songs::lyrics::get_lyrics,
        crate::routes::performances::list_performances,
        crate::routes::performances::get_performance,
        crate::routes::performances::lyrics::get_lyrics,
    ),
    components(schemas(
        SongSummary,
        SongResponse,
        PerformanceSummary,
        PerformanceResponse,
        LyricsResponse,
        ArtistInfo,
        TagInfo,
        ImageInfo,
        MediaInfo,
        ErrorResponse,
    )),
    info(title = "Bajiru Karaoke API", version = "0.1.0"),
    tags(
        (name = "songs", description = "Song catalog"),
        (name = "performances", description = "Karaoke performances"),
    )
)]
struct PublicApiDoc;

/// Full API Docs (Admin only)
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::songs::create_song,
        crate::routes::songs::update_song,
        crate::routes::songs::delete_song,
        crate::routes::songs::lyrics::put_lyrics,
        crate::routes::songs::lyrics::delete_lyrics,
        crate::routes::performances::create_performance,
        crate::routes::performances::update_performance,
        crate::routes::performances::delete_performance,
        crate::routes::performances::upload_audio,
        crate::routes::performances::delete_audio,
        crate::routes::performances::upload_video,
        crate::routes::performances::delete_video,
        crate::routes::performances::lyrics::put_lyrics,
        crate::routes::performances::lyrics::delete_lyrics,
    ),
    components(schemas(
        CreateSongRequest,
        UpdateSongRequest,
        UpdateLyricsRequest,
        CreatePerformanceRequest,
        UpdatePerformanceRequest,
        FileUpload,
    )),
    info(title = "Bajiru Karaoke API — Full", version = "0.1.0"),
    tags(
        (name = "songs", description = "Song catalog"),
        (name = "performances", description = "Karaoke performances"),
    )
)]
struct AdminApiDoc;

pub fn router() -> Router<AppState> {
    let mut full_spec = AdminApiDoc::openapi();
    full_spec.merge(PublicApiDoc::openapi());

    Router::new().merge(
        SwaggerUi::new("/docs")
            .url("/docs/openapi.json", PublicApiDoc::openapi())
            .url("/docs/admin/openapi.json", full_spec),
    )
}
