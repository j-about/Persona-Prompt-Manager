//! Token Management Commands
//!
//! This module provides Tauri IPC commands for managing tokens, which are the atomic
//! units of image generation prompts. Tokens are organized by:
//!
//! - **Granularity Level**: Hierarchical categories (Style, General, Hair, Face, Upper Body, Midsection, Lower Body)
//! - **Polarity**: Whether the token describes desired (positive) or undesired (negative) characteristics
//! - **Weight**: Relative importance/emphasis in the final prompt (0.8 to 1.5 typically)
//!
//! # Token Organization
//!
//! Tokens are grouped by granularity level to enable selective prompt composition.
//! Users can choose which levels to include when composing prompts, allowing for
//! flexible reuse of persona definitions.

use tauri::State;

use crate::domain::token::{
    BatchCreateTokenRequest, CreateTokenRequest, GranularityLevel, ReorderTokensRequest, Token,
    UpdateTokenRequest,
};
use crate::error::AppError;
use crate::infrastructure::database::repositories::TokenRepository;
use crate::AppState;

/// Creates a single token for a persona.
///
/// The token is automatically assigned the next global display order within
/// the persona, appearing at the end of the token list.
///
/// # Arguments
///
/// * `state` - Application state containing the database connection
/// * `request` - Token creation data including `persona_id`, `granularity_id`, polarity, content, and weight
///
/// # Returns
///
/// The newly created token with generated ID and timestamps.
#[tauri::command]
pub fn create_token(
    state: State<AppState>,
    request: CreateTokenRequest,
) -> Result<Token, AppError> {
    let db = state
        .db
        .lock()
        .map_err(|_| AppError::Internal("Failed to acquire database lock".to_string()))?;

    TokenRepository::create(db.connection(), &request)
}

/// Creates multiple tokens at once from comma-separated input.
///
/// This is the primary method for adding tokens, allowing users to quickly enter
/// multiple tokens in a single operation. Each comma-separated value becomes a
/// separate token with the same granularity, polarity, and weight.
///
/// # Arguments
///
/// * `state` - Application state containing the database connection
/// * `request` - Batch creation data with comma-separated contents string
///
/// # Returns
///
/// Vector of all newly created tokens, in creation order.
///
/// # Example
///
/// A request with contents "red hair, long hair, flowing" creates three tokens.
#[tauri::command]
pub fn create_tokens_batch(
    state: State<AppState>,
    request: BatchCreateTokenRequest,
) -> Result<Vec<Token>, AppError> {
    let db = state
        .db
        .lock()
        .map_err(|_| AppError::Internal("Failed to acquire database lock".to_string()))?;

    let contents = request.parse_contents();

    TokenRepository::create_batch(
        db.connection(),
        &request.persona_id,
        &request.granularity_id,
        request.polarity,
        &contents,
        request.weight,
    )
}

/// Retrieves all tokens for a persona in user-defined order.
///
/// Tokens are returned ordered by global `display_order` which reflects
/// the user's drag-and-drop arrangement.
///
/// # Arguments
///
/// * `state` - Application state containing the database connection
/// * `persona_id` - UUID of the persona whose tokens to retrieve
///
/// # Returns
///
/// Vector of all tokens belonging to the persona, which may be empty.
#[tauri::command]
pub fn get_tokens_by_persona(
    state: State<AppState>,
    persona_id: String,
) -> Result<Vec<Token>, AppError> {
    let db = state
        .db
        .lock()
        .map_err(|_| AppError::Internal("Failed to acquire database lock".to_string()))?;

    TokenRepository::find_by_persona(db.connection(), &persona_id)
}

/// Updates a token's content, weight, granularity, or polarity.
///
/// Only fields present in the request are updated. The `updated_at` timestamp
/// is automatically refreshed.
///
/// # Arguments
///
/// * `state` - Application state containing the database connection
/// * `id` - UUID of the token to update
/// * `request` - Partial update data (all fields optional)
///
/// # Returns
///
/// The updated token with all current field values.
///
/// # Errors
///
/// Returns `AppError::NotFound` if no token exists with the given ID.
#[tauri::command]
pub fn update_token(
    state: State<AppState>,
    id: String,
    request: UpdateTokenRequest,
) -> Result<Token, AppError> {
    let db = state
        .db
        .lock()
        .map_err(|_| AppError::Internal("Failed to acquire database lock".to_string()))?;

    TokenRepository::update(db.connection(), &id, &request)
}

/// Deletes a token permanently.
///
/// # Arguments
///
/// * `state` - Application state containing the database connection
/// * `id` - UUID of the token to delete
///
/// # Errors
///
/// Returns `AppError::NotFound` if no token exists with the given ID.
#[tauri::command]
pub fn delete_token(state: State<AppState>, id: String) -> Result<(), AppError> {
    let db = state
        .db
        .lock()
        .map_err(|_| AppError::Internal("Failed to acquire database lock".to_string()))?;

    TokenRepository::delete(db.connection(), &id)
}

/// Returns all available granularity levels.
///
/// Granularity levels are hardcoded constants representing the hierarchical
/// categories for organizing tokens: Style, General, Hair, Face, Upper Body, Midsection, Lower Body.
///
/// This endpoint provides the frontend with the canonical list for UI rendering
/// and validation.
///
/// # Returns
///
/// Vector of all granularity levels in display order.
#[tauri::command]
#[must_use]
pub fn get_all_granularity_levels() -> Vec<GranularityLevel> {
    GranularityLevel::all()
}

/// Reorders tokens within a persona.
///
/// Accepts a batch of token ID to display_order mappings and updates all
/// positions atomically. The frontend computes the complete new ordering
/// after drag-and-drop operations.
///
/// # Arguments
///
/// * `state` - Application state containing the database connection
/// * `request` - Reorder request with `persona_id` and `token_orders` array
///
/// # Errors
///
/// Returns `AppError::Validation` if any token doesn't belong to the specified persona.
/// Returns `AppError::NotFound` if any token ID doesn't exist.
#[tauri::command]
pub fn reorder_tokens(
    state: State<AppState>,
    request: ReorderTokensRequest,
) -> Result<(), AppError> {
    let db = state
        .db
        .lock()
        .map_err(|_| AppError::Internal("Failed to acquire database lock".to_string()))?;

    TokenRepository::reorder_tokens(db.connection(), &request)
}
