use std::sync::Arc;

use cookie::{Cookie, SameSite};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

const SESSION_SECS: i64 = 30 * 24 * 60 * 60;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: u32,
    pub caps: Vec<String>,
    pub exp: usize,
}

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

pub fn verify(token: &str, key: &Arc<DecodingKey>) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(token, key, &Validation::default()).map(|d| d.claims)
}

pub fn session_cookie(token: String) -> Cookie<'static> {
    let mut c = Cookie::new("session", token);
    c.set_http_only(true);
    c.set_same_site(SameSite::Lax);
    c.set_path("/");
    c.set_secure(true);
    c
}
