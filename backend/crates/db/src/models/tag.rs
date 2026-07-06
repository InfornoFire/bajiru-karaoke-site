//! Tag model.

use serde::{Deserialize, Serialize};

/// A tag record fetched from the database.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Tag {
    pub id: u32,
    pub name: String,
    /// Freeform category string (e.g. `"genre"`, `"mood"`).
    pub kind: String,
}

/// Input for creating or looking up a tag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTag {
    pub name: String,
    pub kind: String,
}
