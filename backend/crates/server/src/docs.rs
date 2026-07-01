use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    auth::AuthApi,
    routes::{performances::PerformancesApi, songs::SongsApi},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    let mut spec = SongsApi::openapi();
    spec.merge(PerformancesApi::openapi());
    spec.merge(AuthApi::openapi());

    Router::new().merge(SwaggerUi::new("/docs").url("/docs/openapi.json", spec))
}
