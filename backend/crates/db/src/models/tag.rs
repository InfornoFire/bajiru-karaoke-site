//! Tag model.

use serde::{Deserialize, Serialize};

/// A tag record from the `tags` table.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Tag {
    pub id: u32,
    pub name: String,
}

/// A tag joined with its kind from an entity-specific join table.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct TagWithKind {
    pub id: u32,
    pub name: String,
    pub kind: String,
}

/// Input for creating or looking up a tag by name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTag {
    pub name: String,
}
