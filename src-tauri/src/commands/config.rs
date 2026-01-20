//! Configuration Commands
//!
//! This module provides Tauri IPC commands for retrieving application configuration
//! constants. These commands expose Rust-defined constants to the TypeScript frontend,
//! ensuring a single source of truth for configuration values.
//!
//! # Design Philosophy
//!
//! Rather than duplicating configuration values in both Rust and TypeScript,
//! the frontend retrieves constants from the backend via these IPC commands.
//! This architecture:
//! - Eliminates synchronization issues between frontend and backend
//! - Makes the Rust backend the authoritative source for all configuration
//! - Allows configuration changes to propagate without frontend modifications
//!
//! # Available Commands
//!
//! - [`get_default_image_model_id`] - Default model for image generation

use crate::domain::DEFAULT_IMAGE_MODEL_ID;

// ============================================================================
// Image Generation Configuration
// ============================================================================

/// Returns the default image generation model identifier.
///
/// This command exposes the [`DEFAULT_IMAGE_MODEL_ID`] constant to the frontend,
/// ensuring both layers use the same default value without duplication.
///
/// # Returns
///
/// The `HuggingFace` model identifier string for Stable Diffusion XL Base 1.0.
///
/// # Example (TypeScript)
///
/// ```typescript
/// const modelId = await invoke<string>('get_default_image_model_id');
/// // Returns: "stabilityai/stable-diffusion-xl-base-1.0"
/// ```
///
/// # See Also
///
/// - [`crate::domain::constants::DEFAULT_IMAGE_MODEL_ID`] - The underlying constant
#[tauri::command]
#[must_use]
pub const fn get_default_image_model_id() -> &'static str {
    DEFAULT_IMAGE_MODEL_ID
}
