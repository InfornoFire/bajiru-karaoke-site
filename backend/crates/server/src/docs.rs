use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    routes::{performances::PerformancesApi, songs::SongsApi},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    let mut spec = SongsApi::openapi();
    spec.merge(PerformancesApi::openapi());

    Router::new().merge(SwaggerUi::new("/docs").url("/docs/openapi.json", spec))
}
