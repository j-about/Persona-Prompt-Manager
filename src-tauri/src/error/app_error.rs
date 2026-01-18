//! Application Error Types
//!
//! This module defines `AppError`, the unified error type used throughout the application.
//! All errors are converted to this type before being returned to the frontend via Tauri IPC.
//!
//! # Error Categories
//!
//! - **Database**: `SQLite` operation failures
//! - **`NotFound`**: Entity lookup failures
//! - **Validation**: Input validation failures
//! - **Io**: File system errors
//! - **Serialization**: JSON parsing errors
//! - **Internal**: Unexpected internal errors
//!
//! # Tauri Compatibility
//!
//! `AppError` implements `Serialize` to enable passing error information
//! to the frontend. The error message is serialized as a string.

use serde::Serialize;
use thiserror::Error;

/// Unified application error type.
///
/// This enum captures all error conditions that can occur in the application,
/// providing consistent error handling and user-friendly messages.
#[derive(Debug, Error)]
pub enum AppError {
    /// Database operation failed (connection, query, constraint violation)
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// Requested entity was not found in the database
    #[error("Not found: {0}")]
    NotFound(String),

    /// Input validation failed (invalid data, duplicate names, etc.)
    #[error("Validation error: {0}")]
    Validation(String),

    /// File system operation failed
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization or deserialization failed
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Unexpected internal error (mutex poisoning, etc.)
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Implements `Serialize` for Tauri IPC compatibility.
///
/// Errors are serialized as their display string, which provides
/// user-friendly error messages to the frontend.
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self::Internal(err.to_string())
    }
}
