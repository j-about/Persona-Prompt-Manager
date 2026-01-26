//! Database Migrations
//!
//! This module handles schema versioning and migration for the `SQLite` database.
//! Migrations are run automatically on application startup to ensure the database
//! schema is up to date.
//!
//! # Migration Strategy
//!
//! The application uses a simple version-based migration system:
//! 1. Check current schema version from `schema_version` table
//! 2. Run any migrations newer than the current version
//! 3. Update the version number on successful completion
//!
//! # Current Schema (v1)
//!
//! ## Tables
//!
//! - **personas**: Core persona entities with name, description, tags, and AI config
//! - **`generation_params`**: Image generation settings (1:1 relationship via FK)
//! - **tokens**: Prompt tokens with granularity, polarity, weights, and ordering
//!
//! ## Constraints
//!
//! - Persona names must be unique
//! - Tokens have a composite unique constraint (`persona_id`, `granularity_id`, polarity, content)
//! - Foreign keys cascade deletes from personas to params and tokens

use rusqlite::Connection;

use crate::error::AppError;

/// Current schema version. Increment when adding new migrations.
pub const SCHEMA_VERSION: i32 = 1;

/// Returns the current schema version for this application.
#[must_use]
pub const fn current_schema_version() -> i32 {
    SCHEMA_VERSION
}

/// Reads the schema version from an existing database connection.
///
/// Returns `None` if the `schema_version` table doesn't exist or is empty,
/// indicating the database is not a valid Persona Prompt Manager database.
///
/// # Arguments
///
/// * `conn` - Reference to the `SQLite` connection to check
///
/// # Errors
///
/// Returns `AppError::Database` if a query fails unexpectedly.
pub fn read_schema_version(conn: &Connection) -> Result<Option<i32>, AppError> {
    // Check if schema_version table exists
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='schema_version'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !table_exists {
        return Ok(None);
    }

    let version: Option<i32> = conn
        .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
            row.get(0)
        })
        .ok();

    Ok(version)
}

/// Runs all pending migrations to bring the schema up to date.
///
/// This function is idempotent - running it multiple times has no effect
/// if the schema is already at the current version.
///
/// # Arguments
///
/// * `conn` - Reference to the `SQLite` connection
///
/// # Errors
///
/// Returns `AppError::Database` if any migration fails.
pub fn run_migrations(conn: &Connection) -> Result<(), AppError> {
    let current_version = get_schema_version(conn)?;

    if current_version < SCHEMA_VERSION {
        if current_version < 1 {
            migrate_v1(conn)?;
        }

        set_schema_version(conn, SCHEMA_VERSION)?;
    }

    Ok(())
}

/// Retrieves the current schema version from the database.
///
/// Creates the `schema_version` table if it doesn't exist, enabling
/// fresh databases to start at version 0.
fn get_schema_version(conn: &Connection) -> Result<i32, AppError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (version INTEGER PRIMARY KEY)",
        [],
    )?;

    let version: Option<i32> = conn
        .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
            row.get(0)
        })
        .ok();

    Ok(version.unwrap_or(0))
}

/// Updates the schema version in the database.
fn set_schema_version(conn: &Connection, version: i32) -> Result<(), AppError> {
    conn.execute("DELETE FROM schema_version", [])?;
    conn.execute(
        "INSERT INTO schema_version (version) VALUES (?1)",
        [version],
    )?;
    Ok(())
}

/// Migration v1: Initial consolidated schema.
///
/// Creates all tables and indexes for the application's core data model.
/// This is a consolidated migration representing the initial release schema.
fn migrate_v1(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        r"
        -- Personas: Core entity storing character profile metadata
        CREATE TABLE IF NOT EXISTS personas (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            tags TEXT NOT NULL DEFAULT '[]',
            ai_provider_id TEXT,
            ai_model_id TEXT,
            ai_instructions TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_personas_name ON personas(name);
        CREATE INDEX IF NOT EXISTS idx_personas_created_at ON personas(created_at);

        -- Generation parameters: Image generation settings (1:1 with personas)
        CREATE TABLE IF NOT EXISTS generation_params (
            persona_id TEXT PRIMARY KEY NOT NULL,
            model_id TEXT NOT NULL,
            seed INTEGER NOT NULL,
            steps INTEGER NOT NULL,
            cfg_scale REAL NOT NULL,
            sampler TEXT,
            scheduler TEXT,
            FOREIGN KEY (persona_id) REFERENCES personas(id) ON DELETE CASCADE
        );

        -- Tokens: Atomic prompt elements with granularity and polarity
        CREATE TABLE IF NOT EXISTS tokens (
            id TEXT PRIMARY KEY NOT NULL,
            persona_id TEXT NOT NULL,
            granularity_id TEXT NOT NULL,
            polarity TEXT NOT NULL,
            content TEXT NOT NULL,
            weight REAL,
            display_order INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (persona_id) REFERENCES personas(id) ON DELETE CASCADE,
            UNIQUE (persona_id, granularity_id, polarity, content)
        );

        CREATE INDEX IF NOT EXISTS idx_tokens_persona_id ON tokens(persona_id);
        CREATE INDEX IF NOT EXISTS idx_tokens_granularity ON tokens(granularity_id);
        CREATE INDEX IF NOT EXISTS idx_tokens_polarity ON tokens(polarity);
        CREATE INDEX IF NOT EXISTS idx_tokens_order ON tokens(persona_id, granularity_id, polarity, display_order);
        ",
    )?;

    Ok(())
}
