mod config;
mod docs;
mod error;
mod media;
mod routes;
mod state;
mod storage;

use state::AppState;
use storage::FileStore;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=info,tower_http=info".parse().unwrap()),
        )
        .init();

    let config = config::Config::from_env().expect("invalid configuration");
    let pool = db::connect(&config.database_url)
        .await
        .expect("failed to connect to database");
    let store = FileStore::new(&config.storage_path, &config.storage_base_url);
    let state = AppState {
        pool,
        store,
        config: config.clone(),
    };
    let app = routes::build_router(state);
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind");
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await.expect("server error");
}
