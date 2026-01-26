//! Persona Management Commands
//!
//! This module provides Tauri IPC commands for managing personas, which are the core
//! organizational unit for AI image generation prompts. Each persona represents a
//! distinct character or concept with associated tokens and generation parameters.
//!
//! # Operations
//!
//! - **CRUD**: Create, read, update, and delete personas
//! - **Duplication**: Clone personas with automatic name deduplication
//! - **Generation Params**: Configure image generation settings per persona

use tauri::State;

use crate::domain::persona::{
    CreatePersonaRequest, GenerationParams, Persona, UpdatePersonaRequest,
};
use crate::error::AppError;
use crate::infrastructure::database::repositories::PersonaRepository;
use crate::AppState;

/// Creates a new persona with the given name, description, and tags.
///
/// This command validates that the persona name is unique before creating. A new UUID
/// is generated for the persona, and default generation parameters are automatically
/// created (linked via foreign key).
///
/// # Arguments
///
/// * `state` - Application state containing the database connection
/// * `request` - Persona creation data (name required, description and tags optional)
///
/// # Returns
///
/// The newly created persona with all fields populated, including generated ID and timestamps.
///
/// # Errors
///
/// Returns `AppError::Validation` if a persona with the same name already exists.
#[tauri::command]
pub fn create_persona(
    state: State<AppState>,
    request: CreatePersonaRequest,
) -> Result<Persona, AppError> {
    let db = state
        .db
        .lock()
        .map_err(|_| AppError::Internal("Failed to acquire database lock".to_string()))?;

    PersonaRepository::create(db.connection(), &request)
}

/// Retrieves a single persona by its unique identifier.
///
/// # Arguments
///
/// * `state` - Application state containing the database connection
/// * `id` - UUID of the persona to retrieve
///
/// # Returns
///
/// The complete persona entity including all metadata and AI configuration.
///
/// # Errors
///
/// Returns `AppError::NotFound` if no persona exists with the given ID.
#[tauri::command]
pub fn get_persona_by_id(state: State<AppState>, id: String) -> Result<Persona, AppError> {
    let db = state
        .db
        .lock()
        .map_err(|_| AppError::Internal("Failed to acquire database lock".to_string()))?;

    PersonaRepository::find_by_id(db.connection(), &id)
}

/// Lists all personas in the database, ordered by creation date (newest first).
///
/// This command returns all personas without pagination. For large datasets,
/// consider using `search_personas` with specific criteria.
///
/// # Arguments
///
/// * `state` - Application state containing the database connection
///
/// # Returns
///
/// Vector of all personas, which may be empty if none exist.
#[tauri::command]
pub fn list_personas(state: State<AppState>) -> Result<Vec<Persona>, AppError> {
    let db = state
        .db
        .lock()
        .map_err(|_| AppError::Internal("Failed to acquire database lock".to_string()))?;

    PersonaRepository::find_all(db.connection())
}

/// Updates an existing persona with the provided field values.
///
/// Only fields present in the request are updated; omitted fields retain their
/// current values. The `updated_at` timestamp is automatically refreshed.
///
/// # Arguments
///
/// * `state` - Application state containing the database connection
/// * `id` - UUID of the persona to update
/// * `request` - Partial update data (all fields optional)
///
/// # Returns
///
/// The updated persona with all current field values.
///
/// # Errors
///
/// Returns `AppError::NotFound` if no persona exists with the given ID.
#[tauri::command]
pub fn update_persona(
    state: State<AppState>,
    id: String,
    request: UpdatePersonaRequest,
) -> Result<Persona, AppError> {
    let db = state
        .db
        .lock()
        .map_err(|_| AppError::Internal("Failed to acquire database lock".to_string()))?;

    PersonaRepository::update(db.connection(), &id, &request)
}

/// Deletes a persona and all associated data.
///
/// This operation cascades to delete related generation parameters and tokens
/// via foreign key constraints. This action is irreversible.
///
/// # Arguments
///
/// * `state` - Application state containing the database connection
/// * `id` - UUID of the persona to delete
///
/// # Errors
///
/// Returns `AppError::NotFound` if no persona exists with the given ID.
#[tauri::command]
pub fn delete_persona(state: State<AppState>, id: String) -> Result<(), AppError> {
    let db = state
        .db
        .lock()
        .map_err(|_| AppError::Internal("Failed to acquire database lock".to_string()))?;

    PersonaRepository::delete(db.connection(), &id)
}

/// Retrieves the image generation parameters for a persona.
///
/// Generation parameters include model selection, seed, steps, CFG scale,
/// sampler, and scheduler settings used when generating images.
///
/// # Arguments
///
/// * `state` - Application state containing the database connection
/// * `persona_id` - UUID of the persona
///
/// # Returns
///
/// The generation parameters associated with the persona.
///
/// # Errors
///
/// Returns `AppError::NotFound` if no persona exists with the given ID.
#[tauri::command]
pub fn get_persona_generation_params(
    state: State<AppState>,
    persona_id: String,
) -> Result<GenerationParams, AppError> {
    let db = state
        .db
        .lock()
        .map_err(|_| AppError::Internal("Failed to acquire database lock".to_string()))?;

    PersonaRepository::find_generation_params(db.connection(), &persona_id)
}

/// Updates the image generation parameters for a persona.
///
/// All parameter fields are replaced with the provided values.
///
/// # Arguments
///
/// * `state` - Application state containing the database connection
/// * `params` - Complete generation parameters (`persona_id` must match existing persona)
#[tauri::command]
pub fn update_generation_params(
    state: State<AppState>,
    params: GenerationParams,
) -> Result<(), AppError> {
    let db = state
        .db
        .lock()
        .map_err(|_| AppError::Internal("Failed to acquire database lock".to_string()))?;

    PersonaRepository::update_generation_params(db.connection(), &params)
}

/// Creates a duplicate of an existing persona with a unique name.
///
/// The duplication process:
/// 1. Copies all persona metadata (name, description, tags)
/// 2. Copies generation parameters
/// 3. Generates a unique name by appending "(Copy)" or "(Copy N)" if needed
///
/// Note: Tokens are intentionally NOT copied. This allows users to create
/// variations of a persona without inheriting potentially unwanted tokens.
///
/// # Arguments
///
/// * `state` - Application state containing the database connection
/// * `id` - UUID of the persona to duplicate
/// * `new_name` - Optional custom name for the copy (auto-deduplicated if taken)
///
/// # Returns
///
/// The newly created persona copy.
///
/// # Errors
///
/// Returns `AppError::NotFound` if the source persona does not exist.
#[tauri::command]
pub fn duplicate_persona(
    state: State<AppState>,
    id: String,
    new_name: Option<String>,
) -> Result<Persona, AppError> {
    let db = state
        .db
        .lock()
        .map_err(|_| AppError::Internal("Failed to acquire database lock".to_string()))?;

    let conn = db.connection();

    let original = PersonaRepository::find_by_id(conn, &id)?;

    // Generate a unique name by incrementing a counter if necessary
    let base_name = new_name.unwrap_or_else(|| format!("{} (Copy)", original.name));
    let mut name = base_name.clone();
    let mut counter = 1;

    while PersonaRepository::name_exists(conn, &name, None)? {
        counter += 1;
        name = format!("{base_name} ({counter})");
    }

    let request = CreatePersonaRequest {
        name,
        description: original.description,
        tags: original.tags,
    };

    let new_persona = PersonaRepository::create(conn, &request)?;

    // Copy generation params to the new persona
    let mut params = PersonaRepository::find_generation_params(conn, &id)?;
    params.persona_id = new_persona.id.clone();
    PersonaRepository::update_generation_params(conn, &params)?;

    Ok(new_persona)
}
