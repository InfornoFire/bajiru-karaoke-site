//! Axum extractor for authenticated requests.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;

use crate::{error::ApiError, state::AppState};

use super::jwt;

/// Axum extractor that requires a valid session cookie.
///
/// Add `auth: AuthUser` as a handler parameter to protect a route. Axum calls
/// [`FromRequestParts`] before the handler body runs; an invalid or missing
/// cookie returns `401` before the handler is reached.
pub struct AuthUser {
    pub user_id: u32,
    pub capabilities: Vec<String>,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state).await.unwrap();
        let token = jar
            .get("session")
            .map(|c| c.value().to_owned())
            .ok_or(ApiError::Unauthorized)?;
        let claims =
            jwt::verify(&token, &state.jwt_decoding_key).map_err(|_| ApiError::Unauthorized)?;
        Ok(AuthUser {
            user_id: claims.sub,
            capabilities: claims.caps,
        })
    }
}
