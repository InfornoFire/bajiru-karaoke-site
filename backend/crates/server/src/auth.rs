pub(crate) mod discord;
pub(crate) mod jwt;
pub(crate) mod middleware;
pub(crate) mod password;
pub(crate) mod twitch;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use axum_extra::extract::CookieJar;
use cookie::Cookie;

use api_types::auth::MeResponse;
use db::queries;

use crate::{error::ApiError, state::AppState};
use middleware::AuthUser;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(password::register))
        .route("/login", post(password::login))
        .route("/twitch", get(twitch::initiate))
        .route("/twitch/callback", get(twitch::callback))
        .route("/discord", get(discord::initiate))
        .route("/discord/callback", get(discord::callback))
        .route("/me", get(me))
        .route("/logout", post(logout))
}

async fn me(State(state): State<AppState>, auth: AuthUser) -> Result<Json<MeResponse>, ApiError> {
    let user = queries::users::get_by_id(&state.pool, auth.user_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(MeResponse {
        id: user.id,
        username: user.username,
        capabilities: auth.capabilities,
    }))
}

async fn logout(jar: CookieJar) -> (CookieJar, StatusCode) {
    let mut removal = Cookie::new("session", "");
    removal.set_path("/");
    (jar.remove(removal), StatusCode::NO_CONTENT)
}
