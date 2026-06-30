pub(crate) mod performances;
pub(crate) mod songs;

use axum::{
    Router,
    http::{HeaderValue, Method, header::CONTENT_TYPE},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{auth, docs, state::AppState};

pub fn build_router(state: AppState) -> Router {
    let cors_origin = state
        .config
        .frontend_url
        .parse::<HeaderValue>()
        .expect("FRONTEND_URL must be a valid HTTP origin");
    let cors = CorsLayer::new()
        .allow_origin(cors_origin)
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([CONTENT_TYPE]);
    Router::new()
        .nest("/api/songs", songs::router())
        .nest("/api/performances", performances::router())
        .nest("/auth", auth::router())
        .merge(docs::router())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
