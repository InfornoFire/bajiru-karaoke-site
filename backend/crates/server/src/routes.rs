pub(crate) mod performances;
pub(crate) mod songs;

use axum::Router;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{docs, state::AppState};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .nest("/api/songs", songs::router())
        .nest("/api/performances", performances::router())
        .merge(docs::router())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
