use db::MySqlPool;

use crate::config::Config;
use crate::storage::FileStore;

#[derive(Clone)]
pub struct AppState {
    pub pool: MySqlPool,
    pub store: FileStore,
    #[allow(dead_code)]
    pub config: Config,
}
