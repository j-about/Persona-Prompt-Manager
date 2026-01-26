//! Database Import/Export Commands
//!
//! This module provides Tauri IPC commands for exporting and importing
//! the entire `SQLite` database file, enabling backup and data migration.
//!
//! # Export Behavior
//!
//! Export performs a WAL checkpoint to ensure all data is written to the
//! main database file, then copies it to the user-selected location.
//!
//! # Import Behavior
//!
//! Import validates the schema version of the imported database, then
//! replaces the current database file. The application database connection
//! is reopened after import.
//!
//! # Schema Validation
//!
//! Before importing, the schema version is validated:
//! - Missing `schema_version` table: Not a valid PPM database
//! - Schema version > current: Incompatible future version (requires app update)

use std::fs;
use std::path::Path;

use rusqlite::Connection;
use tauri::State;
use tauri_plugin_dialog::DialogExt;

use crate::domain::export::{ExportResult, ImportResult};
use crate::error::AppError;
use crate::infrastructure::database::migrations::{current_schema_version, read_schema_version};
use crate::infrastructure::Database;
use crate::AppState;

/// Exports the database to a user-selected location.
///
/// Performs WAL checkpoint before export to ensure data integrity.
/// Opens a native save dialog for the user to choose the destination.
///
/// # Arguments
///
/// * `app` - Tauri application handle for dialog access
/// * `state` - Application state containing the database connection and path
///
/// # Returns
///
/// `ExportResult` indicating success with file path, failure with error,
/// or cancellation (success=false, error=None).
#[tauri::command]
pub async fn export_database(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<ExportResult, AppError> {
    // Get the database and perform WAL checkpoint
    {
        let db = state
            .db
            .lock()
            .map_err(|_| AppError::Internal("Failed to acquire database lock".to_string()))?;

        let conn = db.connection();

        // Checkpoint WAL to ensure all data is in the main database file
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
    }

    // Show save dialog
    let file_path = app
        .dialog()
        .file()
        .set_title("Export Database")
        .set_file_name(format!(
            "ppm-backup-{}.db",
            chrono::Utc::now().format("%Y-%m-%d")
        ))
        .add_filter("SQLite Database", &["db"])
        .blocking_save_file();

    let Some(file_path) = file_path else {
        return Ok(ExportResult::cancelled());
    };

    let dest_path = file_path.as_path().ok_or_else(|| {
        AppError::Validation("Invalid file path: URL paths are not supported".to_string())
    })?;

    // Copy database file to destination
    fs::copy(&state.db_path, dest_path)?;

    Ok(ExportResult::success(dest_path.to_string_lossy().to_string()))
}

/// Imports a database from a user-selected file.
///
/// Validates schema version before import. Rejects databases with:
/// - No `schema_version` table (not a PPM database)
/// - Schema version higher than current (incompatible future version)
///
/// Replaces the current database and reopens the connection.
///
/// # Arguments
///
/// * `app` - Tauri application handle for dialog access
/// * `state` - Application state containing the database connection and path
///
/// # Returns
///
/// `ImportResult` indicating success with persona count, or failure with error.
#[tauri::command]
pub async fn import_database(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<ImportResult, AppError> {
    // Show open dialog
    let file_path = app
        .dialog()
        .file()
        .set_title("Import Database")
        .add_filter("SQLite Database", &["db"])
        .blocking_pick_file();

    let Some(file_path) = file_path else {
        return Ok(ImportResult::failure("Import cancelled".to_string()));
    };

    let source_path = file_path.as_path().ok_or_else(|| {
        AppError::Validation("Invalid file path: URL paths are not supported".to_string())
    })?;

    // Validate the imported database
    let personas_count = validate_and_count_personas(source_path)?;

    // Close current database connection and replace the file
    {
        let mut db = state
            .db
            .lock()
            .map_err(|_| AppError::Internal("Failed to acquire database lock".to_string()))?;

        // Copy the imported database over the current one
        fs::copy(source_path, &state.db_path)?;

        // Remove any WAL/SHM files from the old database
        let wal_path = state.db_path.with_extension("db-wal");
        let shm_path = state.db_path.with_extension("db-shm");
        let _ = fs::remove_file(wal_path); // Ignore errors if files don't exist
        let _ = fs::remove_file(shm_path);

        // Reopen the database connection
        *db = Database::new(&state.db_path)?;
    }

    Ok(ImportResult::success(personas_count))
}

/// Validates an imported database file.
///
/// Checks:
/// 1. File can be opened as `SQLite` database
/// 2. `schema_version` table exists
/// 3. Schema version is <= current application version
/// 4. `personas` table exists
///
/// Returns the count of personas in the database.
fn validate_and_count_personas(path: &Path) -> Result<usize, AppError> {
    // Open the imported database read-only
    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    // Check schema version
    let schema_version = read_schema_version(&conn)?;

    match schema_version {
        None => {
            return Err(AppError::Validation(
                "Invalid database: not a Persona Prompt Manager database (missing schema version)"
                    .to_string(),
            ));
        }
        Some(version) if version > current_schema_version() => {
            return Err(AppError::Validation(format!(
                "Incompatible database: schema version {} is newer than supported version {}. \
                Please update the application.",
                version,
                current_schema_version()
            )));
        }
        Some(_) => {} // Valid version
    }

    // Count personas
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM personas", [], |row| row.get(0))
        .map_err(|_| {
            AppError::Validation("Invalid database: personas table not found".to_string())
        })?;

    // Safe conversion: COUNT(*) is always non-negative
    Ok(usize::try_from(count).unwrap_or(0))
}
