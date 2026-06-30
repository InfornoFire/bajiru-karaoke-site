use std::sync::Arc;

use db::MySqlPool;
use jsonwebtoken::{DecodingKey, EncodingKey};
use oauth2::basic::BasicClient;

use crate::config::Config;
use crate::storage::FileStore;

#[derive(Clone)]
pub struct AppState {
    pub pool: MySqlPool,
    pub store: FileStore,
    pub config: Config,
    pub twitch_oauth: Arc<BasicClient>,
    pub discord_oauth: Arc<BasicClient>,
    pub jwt_encoding_key: Arc<EncodingKey>,
    pub jwt_decoding_key: Arc<DecodingKey>,
    pub http_client: reqwest::Client,
}
