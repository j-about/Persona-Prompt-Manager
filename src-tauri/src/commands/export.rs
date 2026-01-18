//! Import/Export Commands
//!
//! This module provides Tauri IPC commands for exporting personas to JSON files
//! and importing them back, enabling backup, sharing, and migration workflows.
//!
//! # Export Format
//!
//! Exports use a versioned JSON format (`BulkExport`) containing:
//! - Export metadata (version, timestamp, app identifier)
//! - Array of `PersonaExport` objects, each containing:
//!   - Complete persona data
//!   - Generation parameters
//!   - All tokens with weights and ordering
//!   - Granularity level definitions
//!
//! # Import Behavior
//!
//! Import supports three conflict resolution strategies:
//! - **Skip**: Leave existing persona unchanged
//! - **Rename**: Create new persona with "(Imported)" suffix
//! - **Replace**: Delete existing persona and import the new one

use tauri::State;

use crate::domain::export::{
    BulkExport, ImportConflictStrategy, ImportOptions, ImportResult, PersonaExport,
};
use crate::domain::persona::CreatePersonaRequest;
use crate::domain::token::{Granularity, GranularityLevel};
use crate::error::AppError;
use crate::infrastructure::database::repositories::{PersonaRepository, TokenRepository};
use crate::AppState;

/// Exports all personas with their complete data to a structured JSON format.
///
/// The export includes everything needed to fully recreate the personas:
/// - Persona metadata (name, description, tags)
/// - Generation parameters (model, seed, steps, etc.)
/// - All tokens with their granularity, polarity, weights, and ordering
/// - Granularity level definitions for compatibility validation
///
/// # Arguments
///
/// * `state` - Application state containing the database connection
///
/// # Returns
///
/// `BulkExport` containing all personas, ready for JSON serialization.
/// The frontend handles downloading this as a file.
#[tauri::command]
pub fn export_all_personas(state: State<AppState>) -> Result<BulkExport, AppError> {
    let db = state
        .db
        .lock()
        .map_err(|_| AppError::Internal("Failed to acquire database lock".to_string()))?;

    let conn = db.connection();

    let personas = PersonaRepository::find_all(conn)?;
    let granularity_levels = GranularityLevel::all();

    let mut exports = Vec::new();

    for persona in personas {
        let generation_params = PersonaRepository::find_generation_params(conn, &persona.id)?;
        let tokens = TokenRepository::find_by_persona(conn, &persona.id)?;

        exports.push(PersonaExport::new(
            persona,
            generation_params,
            tokens,
            granularity_levels.clone(),
        ));
    }

    Ok(BulkExport::new(exports))
}

/// Imports a single persona with conflict handling.
///
/// This internal helper handles the import logic for one persona, including
/// name conflict resolution and token validation against known granularity levels.
///
/// # Arguments
///
/// * `state` - Application state containing the database connection
/// * `export` - The persona export data to import
/// * `options` - Import behavior settings including conflict strategy
///
/// # Returns
///
/// `ImportResult` indicating success/failure with details about what was imported
/// and any warnings encountered.
fn import_persona(
    state: State<AppState>,
    export: PersonaExport,
    options: ImportOptions,
) -> Result<ImportResult, AppError> {
    let db = state
        .db
        .lock()
        .map_err(|_| AppError::Internal("Failed to acquire database lock".to_string()))?;

    let conn = db.connection();

    let mut warnings = Vec::new();

    // Handle name conflicts based on the selected strategy
    let name_exists = PersonaRepository::name_exists(conn, &export.persona.name, None)?;

    let persona_name = if name_exists {
        match options.on_conflict {
            ImportConflictStrategy::Skip => {
                return Ok(ImportResult::failure(format!(
                    "Persona '{}' already exists",
                    export.persona.name
                )));
            }
            ImportConflictStrategy::Rename => {
                // Generate unique name with incrementing suffix
                let mut new_name = format!("{} (Imported)", export.persona.name);
                let mut counter = 1;
                while PersonaRepository::name_exists(conn, &new_name, None)? {
                    counter += 1;
                    new_name = format!("{} (Imported {})", export.persona.name, counter);
                }
                warnings.push(format!(
                    "Renamed from '{}' to '{}'",
                    export.persona.name, new_name
                ));
                new_name
            }
            ImportConflictStrategy::Replace => {
                // Delete existing persona before importing
                let existing = PersonaRepository::find_all(conn)?
                    .into_iter()
                    .find(|p| p.name == export.persona.name);
                if let Some(existing) = existing {
                    PersonaRepository::delete(conn, &existing.id)?;
                    warnings.push(format!(
                        "Replaced existing persona '{}'",
                        export.persona.name
                    ));
                }
                export.persona.name.clone()
            }
        }
    } else {
        export.persona.name.clone()
    };

    // Create the new persona
    let create_request = CreatePersonaRequest {
        name: persona_name,
        description: export.persona.description,
        tags: export.persona.tags,
    };

    let new_persona = PersonaRepository::create(conn, &create_request)?;

    // Copy generation parameters to the new persona
    let mut params = export.generation_params.clone();
    params.persona_id = new_persona.id.clone();
    PersonaRepository::update_generation_params(conn, &params)?;

    // Import tokens, validating granularity levels
    let mut tokens_imported = 0;
    for token in &export.tokens {
        // Only import tokens with valid granularity levels
        if Granularity::parse(&token.granularity_id).is_some() {
            TokenRepository::create_batch(
                conn,
                &new_persona.id,
                &token.granularity_id,
                token.polarity,
                std::slice::from_ref(&token.content),
                token.weight,
            )?;
            tokens_imported += 1;
        } else {
            warnings.push(format!(
                "Skipped token '{}': unknown granularity level '{}'",
                token.content, token.granularity_id
            ));
        }
    }

    Ok(ImportResult::success(
        new_persona,
        tokens_imported,
        warnings,
    ))
}

/// Imports multiple personas from a bulk export.
///
/// Each persona is imported independently, so failures for one persona don't
/// affect others. The database lock is released between personas to avoid
/// holding it for extended periods during large imports.
///
/// # Arguments
///
/// * `state` - Application state containing the database connection
/// * `export` - The bulk export data containing multiple personas
/// * `options` - Import behavior settings (applied to all personas)
///
/// # Returns
///
/// Vector of `ImportResult`, one per persona in the export, in the same order.
#[tauri::command]
pub fn import_personas(
    state: State<AppState>,
    export: BulkExport,
    options: ImportOptions,
) -> Result<Vec<ImportResult>, AppError> {
    let mut results = Vec::new();

    for persona_export in export.personas {
        match import_persona(state.clone(), persona_export, options.clone()) {
            Ok(result) => results.push(result),
            Err(e) => results.push(ImportResult::failure(e.to_string())),
        }
    }

    Ok(results)
}

/// Parses JSON input into a `BulkExport` for import.
///
/// Accepts either:
/// - A `BulkExport` JSON object (multiple personas)
/// - A single `PersonaExport` JSON object (automatically wrapped)
///
/// This flexibility allows users to import from both full backups and
/// individual persona exports.
///
/// # Arguments
///
/// * `json` - JSON string to parse
///
/// # Returns
///
/// `BulkExport` ready for import, or error if JSON is invalid.
#[tauri::command]
pub fn parse_import_json(json: String) -> Result<BulkExport, AppError> {
    // Try bulk export format first
    if let Ok(bulk) = serde_json::from_str::<BulkExport>(&json) {
        return Ok(bulk);
    }

    // Try single persona export and wrap it
    if let Ok(single) = serde_json::from_str::<PersonaExport>(&json) {
        return Ok(BulkExport::new(vec![single]));
    }

    Err(AppError::Validation(
        "Invalid import format. Expected PersonaExport or BulkExport JSON.".to_string(),
    ))
}
