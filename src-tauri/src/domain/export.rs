//! Export/Import Domain Types
//!
//! This module defines the data structures for database export and import,
//! enabling backup and migration of the entire application database.
//!
//! # Export Format
//!
//! The export format is the raw `SQLite` database file (`.db`), providing:
//! - **Complete backup**: All personas, tokens, and settings preserved
//! - **Integrity**: Database constraints and indexes maintained
//! - **Simplicity**: No serialization/deserialization complexity
//!
//! # Schema Validation
//!
//! Before importing, the schema version is validated to prevent importing
//! databases from incompatible versions of the application.

use serde::{Deserialize, Serialize};

/// Result of a database export operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    /// Whether the export completed successfully
    pub success: bool,
    /// Path where the file was saved (present only on success)
    pub path: Option<String>,
    /// Error message (present only on failure)
    pub error: Option<String>,
}

impl ExportResult {
    /// Creates a successful export result with the destination path.
    #[must_use]
    pub const fn success(path: String) -> Self {
        Self {
            success: true,
            path: Some(path),
            error: None,
        }
    }

    /// Creates a failed export result with an error message.
    #[must_use]
    pub const fn failure(error: String) -> Self {
        Self {
            success: false,
            path: None,
            error: Some(error),
        }
    }

    /// Creates a cancelled result (user cancelled the dialog).
    ///
    /// This is not an error condition; `success` is false but `error` is `None`.
    #[must_use]
    pub const fn cancelled() -> Self {
        Self {
            success: false,
            path: None,
            error: None,
        }
    }
}

/// Result of a database import operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    /// Whether the import completed successfully
    pub success: bool,
    /// Number of personas in the imported database
    pub personas_count: usize,
    /// Error message (present only on failure)
    pub error: Option<String>,
}

impl ImportResult {
    /// Creates a successful import result with the persona count.
    #[must_use]
    pub const fn success(personas_count: usize) -> Self {
        Self {
            success: true,
            personas_count,
            error: None,
        }
    }

    /// Creates a failed import result with an error message.
    #[must_use]
    pub const fn failure(error: String) -> Self {
        Self {
            success: false,
            personas_count: 0,
            error: Some(error),
        }
    }
}
