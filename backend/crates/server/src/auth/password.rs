use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use db::{models::NewUser, queries};

use crate::{error::ApiError, state::AppState};

#[derive(Deserialize)]
pub struct RegisterRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

pub async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<RegisterRequest>,
) -> Result<(CookieJar, StatusCode), ApiError> {
    validate_username(&req.username)?;
    validate_password(&req.password)?;

    let hash = hash_password(&req.password)?;

    let user = queries::users::create(
        &state.pool,
        &NewUser {
            username: req.username,
            twitch_id: None,
            discord_id: None,
        },
    )
    .await
    .map_err(|e| match e {
        db::error::DbError::Conflict => ApiError::Conflict("username already taken".into()),
        other => ApiError::from(other),
    })?;

    queries::user_credentials::create(&state.pool, user.id, &hash).await?;

    let caps = queries::capabilities::list_for_user(&state.pool, user.id)
        .await?
        .into_iter()
        .map(|c| c.title)
        .collect();

    let token = super::jwt::issue(user.id, caps, &state.jwt_encoding_key)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok((
        jar.add(super::jwt::session_cookie(token)),
        StatusCode::CREATED,
    ))
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<LoginRequest>,
) -> Result<(CookieJar, StatusCode), ApiError> {
    let user = queries::users::get_by_username(&state.pool, &req.username)
        .await?
        .ok_or(ApiError::Unauthorized)?;

    let cred = queries::user_credentials::get_by_user_id(&state.pool, user.id)
        .await?
        .ok_or(ApiError::Unauthorized)?;

    verify_password(&req.password, &cred.password_hash)?;

    let caps = queries::capabilities::list_for_user(&state.pool, user.id)
        .await?
        .into_iter()
        .map(|c| c.title)
        .collect();

    let token = super::jwt::issue(user.id, caps, &state.jwt_encoding_key)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok((jar.add(super::jwt::session_cookie(token)), StatusCode::OK))
}

fn validate_username(username: &str) -> Result<(), ApiError> {
    if username.is_empty() || username.len() > 64 {
        return Err(ApiError::BadRequest(
            "username must be between 1 and 64 characters".into(),
        ));
    }
    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
    {
        return Err(ApiError::BadRequest(
            "username may only contain letters, numbers, underscores, and periods".into(),
        ));
    }
    Ok(())
}

fn validate_password(password: &str) -> Result<(), ApiError> {
    if password.len() < 8 {
        return Err(ApiError::BadRequest(
            "password must be at least 8 characters".into(),
        ));
    }
    Ok(())
}

fn hash_password(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| ApiError::Internal(e.to_string()))
}

fn verify_password(password: &str, hash: &str) -> Result<(), ApiError> {
    let parsed = PasswordHash::new(hash).map_err(|e| ApiError::Internal(e.to_string()))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| ApiError::Unauthorized)
}
