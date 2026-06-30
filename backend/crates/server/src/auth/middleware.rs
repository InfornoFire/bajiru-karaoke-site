use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;

use crate::{error::ApiError, state::AppState};

use super::jwt;

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
