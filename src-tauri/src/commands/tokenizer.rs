//! Tokenizer Commands
//!
//! This module provides model-aware token counting for validating prompt lengths
//! against the limits of different image generation models.
//!
//! # Why Token Counting Matters
//!
//! Different image generation models have different tokenizers and limits:
//! - **SDXL/SD 1.5**: CLIP tokenizer, 77 tokens max (75 usable)
//! - **PixArt/Hunyuan**: T5 tokenizer, 256 tokens max (250 usable)
//!
//! Exceeding these limits causes prompt truncation, which can silently drop
//! important tokens from the end of prompts.
//!
//! # Tokenizer Selection
//!
//! The system automatically selects the appropriate tokenizer based on the model ID:
//! - Exact match against known model configurations
//! - Family-based fallback (e.g., any "pixart" model uses T5)
//! - Default to CLIP tokenizer for unknown models

use crate::infrastructure::tokenizer::{self, TokenCount, TokenizerInfo};

/// Counts tokens in text for a specific image generation model.
///
/// Uses the `HuggingFace` tokenizers library for accurate counting with the same
/// tokenizer used by the target model. Falls back to word-based estimation
/// if the tokenizer cannot be loaded.
///
/// # Arguments
///
/// * `text` - The prompt text to count tokens for
/// * `model_id` - Optional model identifier (e.g., "stabilityai/stable-diffusion-xl-base-1.0").
///               Defaults to SDXL-compatible CLIP tokenizer if not specified.
///
/// # Returns
///
/// `TokenCount` with:
/// - `count`: Number of tokens in the text
/// - `max_tokens`: Model's absolute token limit
/// - `usable_tokens`: Tokens available after accounting for special tokens
/// - `exceeds_limit`: Whether the prompt is too long
/// - `usage_percent`: Percentage of limit used (can exceed 100%)
#[tauri::command]
#[must_use]
pub fn count_tokens_for_model(text: String, model_id: Option<String>) -> TokenCount {
    tokenizer::count_tokens(&text, model_id.as_deref())
}

/// Returns configuration information for all known image generation models.
///
/// Provides the frontend with the complete list of supported models and their
/// tokenizer configurations for display in model selection UI.
///
/// # Returns
///
/// Vector of `TokenizerInfo` containing:
/// - `model_id`: Full model identifier
/// - `tokenizer_id`: `HuggingFace` tokenizer used
/// - `max_tokens`/`usable_tokens`: Token limits
#[tauri::command]
#[must_use]
pub fn get_known_image_models() -> Vec<TokenizerInfo> {
    tokenizer::get_known_models()
}
