//! Domain Constants
//!
//! This module defines application-wide constants used across the domain layer.
//! These values represent defaults and configuration that serve as the single
//! source of truth for both the Rust backend and the TypeScript frontend.
//!
//! # Design Philosophy
//!
//! Constants defined here are:
//! - Exposed to the frontend via Tauri IPC commands (see [`crate::commands::config`])
//! - Used directly in backend code via `use crate::domain::DEFAULT_IMAGE_MODEL_ID`
//!
//! This architecture ensures consistency between frontend and backend without
//! duplicating values across codebases.
//!
//! # Image Generation Defaults
//!
//! The default image model is Stable Diffusion XL Base 1.0, chosen for:
//! - Wide compatibility with image generation
//! - CLIP tokenizer support (77 tokens max, 75 usable)
//! - Mature ecosystem with predictable behavior

// ============================================================================
// Image Generation Constants
// ============================================================================

/// Default image generation model identifier.
///
/// This is Stable Diffusion XL Base 1.0 from Stability AI, which provides a
/// balanced trade-off between quality, compatibility, and ecosystem support.
///
/// # Tokenizer Configuration
///
/// This model uses the CLIP tokenizer (`openai/clip-vit-large-patch14`) with:
/// - **Max tokens**: 77
/// - **Usable tokens**: 75 (after accounting for special tokens)
///
/// # Usage
///
/// This constant is used as the fallback value when:
/// - No model is specified in token counting operations
/// - Default generation parameters are created for new personas
/// - AI prompt context needs a baseline model configuration
///
/// # Frontend Access
///
/// The frontend retrieves this value via the `get_default_image_model_id`
/// Tauri command, ensuring a single source of truth.
///
/// # See Also
///
/// - [`crate::infrastructure::tokenizer::get_config_for_model`] - Tokenizer configuration lookup
/// - [`crate::infrastructure::tokenizer::get_prompt_context_for_model`] - Prompt engineering context
/// - [`crate::domain::persona::GenerationParams`] - Default generation parameters
pub const DEFAULT_IMAGE_MODEL_ID: &str = "stabilityai/stable-diffusion-xl-base-1.0";
