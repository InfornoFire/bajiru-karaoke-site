//! Tag kind enums for per-entity tag classification.
//!
//! Kind is a property of how a tag is applied to a specific entity, not of the tag
//! itself. Each entity type defines its own valid kind set here, and the database
//! stores the serialized string without a constraint. Invalid values are rejected at
//! deserialization before any query runs.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Valid kind values when applying a tag to a song.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SongTagKind {
    Genre,
    /// Where the song originates from (anime, game, original, vocaloid, etc.).
    Source,
    Language,
    Misc,
}

impl SongTagKind {
    /// Returns the string stored in the database for this kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Genre => "genre",
            Self::Source => "source",
            Self::Language => "language",
            Self::Misc => "misc",
        }
    }
}

/// Valid kind values when applying a tag to a performance.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PerformanceTagKind {
    /// Instrument used in the performance (e.g. ukulele, piano).
    Instrument,
    /// A stylistic or arrangement variation of the song (e.g. uke, slow version).
    Modifier,
    Misc,
}

impl PerformanceTagKind {
    /// Returns the string stored in the database for this kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Instrument => "instrument",
            Self::Modifier => "modifier",
            Self::Misc => "misc",
        }
    }
}
