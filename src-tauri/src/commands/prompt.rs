//! Prompt Composition Commands
//!
//! This module provides the core prompt assembly functionality that transforms
//! a persona's tokens into ready-to-use positive and negative prompts for
//! image generation.
//!
//! # Composition Process
//!
//! 1. Retrieves all tokens for the specified persona
//! 2. Filters tokens by selected granularity levels (or uses all if none specified)
//! 3. Groups tokens by polarity (positive/negative)
//! 4. Applies weight formatting if enabled (e.g., "(token:1.2)")
//! 5. Joins tokens with the configured separator
//! 6. Optionally inserts ad-hoc tokens at the beginning or end

use tauri::State;

use crate::domain::prompt::{ComposedPrompt, CompositionOptions, PromptComposer};
use crate::domain::token::GranularityLevel;
use crate::error::AppError;
use crate::infrastructure::database::repositories::TokenRepository;
use crate::AppState;

/// Composes a prompt from a persona's tokens with configurable options.
///
/// This is the primary endpoint for generating prompts ready for image generation.
/// The composition respects token ordering and applies model-appropriate formatting.
///
/// # Arguments
///
/// * `state` - Application state containing the database connection
/// * `persona_id` - UUID of the persona whose tokens to compose
/// * `options` - Optional composition settings:
///   - `include_weights`: Whether to format tokens with weight modifiers (default: true)
///   - `separator`: String to join tokens (default: ", ")
///   - `granularity_ids`: Which levels to include (default: all, in display order)
///   - `adhoc_positive/negative`: Additional tokens to inject
///   - `adhoc_position`: Where to place ad-hoc tokens (beginning or end)
///
/// # Returns
///
/// A `ComposedPrompt` containing:
/// - `positive_prompt`: Ready-to-use positive prompt string
/// - `negative_prompt`: Ready-to-use negative prompt string
/// - Token counts for both prompts
/// - Breakdown showing which tokens came from which granularity levels
///
/// # Example Output
///
/// With tokens "masterpiece", "1girl", "red hair" and options `include_weights: true`:
/// ```text
/// positive_prompt: "masterpiece, 1girl, (red hair:1.1)"
/// ```
#[tauri::command]
pub fn compose_prompt(
    state: State<AppState>,
    persona_id: String,
    options: Option<CompositionOptions>,
) -> Result<ComposedPrompt, AppError> {
    let db = state
        .db
        .lock()
        .map_err(|_| AppError::Internal("Failed to acquire database lock".to_string()))?;

    let conn = db.connection();

    let tokens = TokenRepository::find_by_persona(conn, &persona_id)?;
    let granularity_levels = GranularityLevel::all();

    let opts = options.unwrap_or_default();
    let composed = PromptComposer::compose(&tokens, &granularity_levels, &opts);

    Ok(composed)
}
