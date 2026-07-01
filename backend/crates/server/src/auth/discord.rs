//! Discord OAuth2 Authorization Code flow handlers.

use axum::{
    extract::{Query, State},
    response::Redirect,
};
use axum_extra::extract::CookieJar;
use cookie::{Cookie, SameSite};
use oauth2::{CsrfToken, Scope};
use serde::Deserialize;

use db::{models::NewUser, queries};

use crate::{error::ApiError, state::AppState};

const DISCORD_TOKEN_URL: &str = "https://discord.com/api/v10/oauth2/token";
const DISCORD_USERS_URL: &str = "https://discord.com/api/v10/users/@me";

#[derive(Deserialize)]
pub struct CallbackParams {
    code: String,
    state: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct DiscordUser {
    id: String,
    username: String,
}

#[utoipa::path(
    get,
    path = "/auth/discord",
    responses(
        (status = 302, description = "Redirect to Discord OAuth, sets CSRF cookie"),
    ),
    tag = "auth"
)]
pub(crate) async fn initiate(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Redirect) {
    let (auth_url, csrf_token) = state
        .discord_oauth
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identify".to_string()))
        .url();

    let mut csrf_cookie = Cookie::new("oauth_csrf_discord", csrf_token.secret().to_owned());
    csrf_cookie.set_http_only(true);
    csrf_cookie.set_same_site(SameSite::Lax);
    csrf_cookie.set_path("/");

    (jar.add(csrf_cookie), Redirect::to(auth_url.as_str()))
}

#[utoipa::path(
    get,
    path = "/auth/discord/callback",
    params(
        ("code" = String, Query, description = "Authorization code from Discord"),
        ("state" = String, Query, description = "CSRF state token"),
    ),
    responses(
        (status = 302, description = "Redirect to frontend, session cookie set"),
        (status = 400, description = "Missing or invalid OAuth state"),
    ),
    tag = "auth"
)]
pub(crate) async fn callback(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(params): Query<CallbackParams>,
) -> Result<(CookieJar, Redirect), ApiError> {
    let csrf_value = jar
        .get("oauth_csrf_discord")
        .map(|c| c.value().to_owned())
        .ok_or_else(|| ApiError::BadRequest("missing OAuth state".to_string()))?;
    if csrf_value != params.state {
        return Err(ApiError::BadRequest("invalid OAuth state".to_string()));
    }

    let redirect_uri = format!("{}/auth/discord/callback", state.config.base_url);
    let token: TokenResponse = state
        .http_client
        .post(DISCORD_TOKEN_URL)
        .form(&[
            ("client_id", state.config.discord_client_id.as_str()),
            ("client_secret", state.config.discord_client_secret.as_str()),
            ("code", params.code.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri.as_str()),
        ])
        .send()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .json()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let discord_user: DiscordUser = state
        .http_client
        .get(DISCORD_USERS_URL)
        .bearer_auth(&token.access_token)
        .send()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .json()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let discord_id: u64 = discord_user
        .id
        .parse()
        .map_err(|_| ApiError::Internal("invalid Discord user ID".to_string()))?;

    let user = queries::users::upsert_by_discord(
        &state.pool,
        &NewUser {
            username: discord_user.username,
            twitch_id: None,
            discord_id: Some(discord_id),
        },
    )
    .await?;

    let caps = queries::capabilities::list_for_user(&state.pool, user.id)
        .await?
        .into_iter()
        .map(|c| c.title)
        .collect();

    let jwt_token = super::jwt::issue(user.id, caps, &state.jwt_encoding_key)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let mut rm_csrf = Cookie::new("oauth_csrf_discord", "");
    rm_csrf.set_path("/");

    Ok((
        jar.remove(rm_csrf)
            .add(super::jwt::session_cookie(jwt_token)),
        Redirect::to(&state.config.frontend_url),
    ))
}
