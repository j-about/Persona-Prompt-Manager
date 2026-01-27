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
//! # Current Schema (v2)
//!
//! ## Tables
//!
//! - **personas**: Core persona entities with name, description, tags, and AI config
//! - **`generation_params`**: Image generation settings (1:1 relationship via FK)
//! - **tokens**: Prompt tokens with granularity, polarity, weights, and global ordering
//!
//! ## v2 Changes
//!
//! - Token `display_order` is now global per persona (not per granularity/polarity group)
//! - Index changed from `(persona_id, granularity_id, polarity, display_order)` to `(persona_id, display_order)`
//!
//! ## Constraints
//!
//! - Persona names must be unique
//! - Tokens have a composite unique constraint (`persona_id`, `granularity_id`, polarity, content)
//! - Foreign keys cascade deletes from personas to params and tokens

use rusqlite::{params, Connection};

use crate::error::AppError;

/// Current schema version. Increment when adding new migrations.
pub const SCHEMA_VERSION: i32 = 2;

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
        if current_version < 2 {
            migrate_v2(conn)?;
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

/// Migration v2: Convert to global token ordering per persona.
///
/// Reassigns `display_order` values to be globally unique within each persona,
/// preserving the logical ordering (by granularity display_order, then polarity, then original display_order).
/// Also updates the index to support the new ordering pattern.
fn migrate_v2(conn: &Connection) -> Result<(), AppError> {
    // Granularity display orders for sorting
    let granularity_order: std::collections::HashMap<&str, i32> = [
        ("style", 0),
        ("general", 1),
        ("hair", 2),
        ("face", 3),
        ("upper_body", 4),
        ("midsection", 5),
        ("lower_body", 6),
    ]
    .into_iter()
    .collect();

    // Get all distinct persona IDs
    let mut stmt = conn.prepare("SELECT DISTINCT persona_id FROM tokens")?;
    let persona_ids: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    for persona_id in &persona_ids {
        // Get tokens in the current logical order (by granularity, polarity, display_order)
        let mut token_stmt = conn.prepare(
            r"SELECT id, granularity_id, polarity, display_order FROM tokens
              WHERE persona_id = ?1
              ORDER BY granularity_id, polarity, display_order",
        )?;

        let mut tokens: Vec<(String, String, String, i32)> = token_stmt
            .query_map([persona_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // Sort by granularity display_order, then polarity (positive first), then original display_order
        tokens.sort_by(|a, b| {
            let gran_order_a = granularity_order.get(a.1.as_str()).unwrap_or(&99);
            let gran_order_b = granularity_order.get(b.1.as_str()).unwrap_or(&99);

            gran_order_a
                .cmp(gran_order_b)
                .then_with(|| a.2.cmp(&b.2)) // polarity: "negative" < "positive" alphabetically, so reverse
                .then_with(|| a.3.cmp(&b.3)) // original display_order
        });

        // Update each token with a new global display_order
        for (new_order, (token_id, _, _, _)) in tokens.iter().enumerate() {
            conn.execute(
                "UPDATE tokens SET display_order = ?1 WHERE id = ?2",
                params![new_order as i32, token_id],
            )?;
        }
    }

    // Update index for new ordering pattern
    conn.execute_batch(
        r"
        CREATE INDEX IF NOT EXISTS idx_personas_updated_at ON personas(updated_at);
        DROP INDEX IF EXISTS idx_tokens_order;
        CREATE INDEX IF NOT EXISTS idx_tokens_global_order ON tokens(persona_id, display_order);
        ",
    )?;

    Ok(())
}
