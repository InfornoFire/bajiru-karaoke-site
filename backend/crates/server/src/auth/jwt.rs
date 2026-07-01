//! JWT issuance and verification for session cookies.
//!
//! Sessions are encoded as HS256 JWTs stored in an HTTP only `session` cookie.
//! The token embeds the user ID and capability list; the server is stateless
//! with respect to auth (no server side session store).

use std::sync::Arc;

use cookie::{Cookie, SameSite};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

const SESSION_SECS: i64 = 30 * 24 * 60 * 60;

/// Claims embedded in the session JWT.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// User ID (`sub` in JWT standard).
    pub sub: u32,
    /// Capability titles granted to this user.
    pub caps: Vec<String>,
    /// Expiry as a Unix timestamp.
    pub exp: usize,
}

/// Signs a new JWT valid for 30 days.
///
/// # Errors
///
/// Returns a [`jsonwebtoken::errors::Error`] if encoding fails.
pub fn issue(
    user_id: u32,
    caps: Vec<String>,
    key: &Arc<EncodingKey>,
) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = (chrono::Utc::now().timestamp() + SESSION_SECS) as usize;
    encode(
        &Header::default(),
        &Claims {
            sub: user_id,
            caps,
            exp,
        },
        key,
    )
}

/// Verifies a JWT and returns its claims.
///
/// # Errors
///
/// Returns a [`jsonwebtoken::errors::Error`] if the token is invalid or expired.
pub fn verify(token: &str, key: &Arc<DecodingKey>) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(token, key, &Validation::default()).map(|d| d.claims)
}

/// Builds the HTTP only `session` cookie carrying the given JWT.
///
/// The cookie has no `Max-Age`, so it is a session cookie from the browser's
/// perspective. Expiry is enforced by the JWT `exp` claim instead.
pub fn session_cookie(token: String) -> Cookie<'static> {
    let mut c = Cookie::new("session", token);
    c.set_http_only(true);
    c.set_same_site(SameSite::Lax);
    c.set_path("/");
    c.set_secure(true);
    c
}
