use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub storage_path: String,
    pub storage_base_url: String,
}

#[derive(Debug)]
pub enum ConfigError {
    Missing(&'static str),
    Invalid(&'static str),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Missing(var) => write!(f, "missing required env var: {var}"),
            ConfigError::Invalid(var) => write!(f, "invalid value for env var: {var}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".into())
            .parse::<u16>()
            .map_err(|_| ConfigError::Invalid("PORT"))?;

        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::Missing("DATABASE_URL"))?,
            port,
            storage_path: env::var("STORAGE_PATH")
                .map_err(|_| ConfigError::Missing("STORAGE_PATH"))?,
            storage_base_url: env::var("STORAGE_BASE_URL")
                .map_err(|_| ConfigError::Missing("STORAGE_BASE_URL"))?,
        })
    }
}
