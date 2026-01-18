//! Database Connection Management
//!
//! Provides `SQLite` database initialization and connection handling.
//! The connection is configured for optimal desktop application performance
//! with WAL mode for crash resilience and concurrent access support.
//!
//! # Initialization Sequence
//!
//! 1. Open or create the database file
//! 2. Enable foreign key constraint enforcement
//! 3. Enable WAL (Write-Ahead Logging) mode
//! 4. Run pending schema migrations

use rusqlite::Connection;
use std::path::Path;

use crate::error::AppError;

use super::migrations;

/// Wrapper around an `SQLite` connection with application-specific configuration.
///
/// This struct owns the database connection and provides access to repositories
/// through the `connection()` method. The connection is configured with:
/// - Foreign key constraints enabled
/// - WAL journal mode for better performance
pub struct Database {
    /// The underlying `SQLite` connection
    pub conn: Connection,
}

impl Database {
    /// Opens or creates a database at the specified path.
    ///
    /// Automatically creates the database file if it doesn't exist,
    /// applies any pending migrations, and configures the connection
    /// for optimal performance.
    ///
    /// # Arguments
    ///
    /// * `path` - File system path for the database file
    ///
    /// # Errors
    ///
    /// Returns `AppError::Database` if the connection fails or migrations error.
    pub fn new(path: &Path) -> Result<Self, AppError> {
        let conn = Connection::open(path)?;

        // Enable foreign key constraints for referential integrity
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        // Enable WAL mode for better concurrent access and crash resilience
        conn.execute_batch("PRAGMA journal_mode = WAL;")?;

        migrations::run_migrations(&conn)?;

        Ok(Self { conn })
    }

    /// Creates an in-memory database for testing.
    ///
    /// The database is initialized with all migrations but no persistent storage.
    /// Data is lost when the connection is dropped.
    #[allow(dead_code)]
    pub fn in_memory() -> Result<Self, AppError> {
        let conn = Connection::open_in_memory()?;

        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        migrations::run_migrations(&conn)?;

        Ok(Self { conn })
    }

    /// Returns a reference to the underlying `SQLite` connection.
    ///
    /// Use this to pass the connection to repository methods.
    pub const fn connection(&self) -> &Connection {
        &self.conn
    }
}
