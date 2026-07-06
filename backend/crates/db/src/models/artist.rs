//! Artist model.

use serde::{Deserialize, Serialize};

/// An artist record fetched from the database.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Artist {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
}

/// Input for creating a new artist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewArtist {
    pub name: String,
    pub description: Option<String>,
}

/// Input for replacing an artist's mutable fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateArtist {
    pub name: String,
    pub description: Option<String>,
}
