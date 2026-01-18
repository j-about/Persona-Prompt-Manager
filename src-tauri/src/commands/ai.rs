//! AI Generation Commands
//!
//! This module provides Tauri IPC commands for AI-powered generation using
//! large language models. Supports multiple AI providers for persona and token generation.
//!
//! # Supported Providers
//!
//! - **`OpenAI`**: gpt-5.2, gpt-5.2-pro, etc.
//! - **Anthropic**: claude-haiku-4-5, claude-sonnet-4-5, claude-opus-4-5
//! - **Google**: gemini-3-flash-preview, gemini-3-pro-preview
//! - **xAI**: grok-4-1-fast-non-reasoning, grok-4-1-fast-reasoning
//! - **Ollama**: Local models (Llama 3.2, etc.) - no API key required

use crate::domain::ai::{
    AiPersonaGenerationRequest, AiPersonaGenerationResponse, AiProvider, AiProviderConfig,
    AiProviderMetadata, TokenGenerationRequest, TokenGenerationResponse,
};
use crate::error::AppError;
use crate::infrastructure::ai;

// ============================================================================
// Persona Generation
// ============================================================================
//
// Creates complete persona profiles with tokens organized by body region.

/// Generates a complete persona using AI, including description, tags, and initial tokens.
///
/// This command takes user inputs (name, style, character description, optional physical
/// criteria) and generates a fully-formed persona profile with tokens organized by
/// granularity level (style, general, hair, face, `upper_body`, midsection, `lower_body`).
///
/// # Arguments
///
/// * `config` - AI provider configuration including provider type, model, and API key
/// * `request` - Generation parameters including:
///   - `name`: Persona name (required)
///   - `style`: Desired visual style (e.g., "anime", "realistic")
///   - `character_description`: Character background and traits
///   - `physical_criteria`: Detailed physical specifications by body region (optional)
///   - `ai_instructions`: Custom instructions for the AI (optional)
///
/// # Returns
///
/// `AiPersonaGenerationResponse` containing:
/// - `description`: Elaborated persona description
/// - `tags`: Inferred tags from style and description
/// - `tokens`: Generated tokens organized by granularity
/// - Provider and model used for attribution
///
/// # Errors
///
/// Returns `AppError::Internal` if the AI request fails or response parsing fails.
#[tauri::command]
pub async fn generate_persona_with_ai(
    config: AiProviderConfig,
    request: AiPersonaGenerationRequest,
) -> Result<AiPersonaGenerationResponse, AppError> {
    ai::generate_persona(&config, &request).await
}

// ============================================================================
// Token Generation
// ============================================================================
//
// Generates additional positive/negative tokens during prompt composition.

/// Generates token suggestions using the configured AI provider.
///
/// This async command sends a request to the specified AI provider and returns
/// structured token suggestions. The prompt is optimized for each provider's
/// strengths (e.g., XML formatting for Claude, JSON mode for GPT).
///
/// # Arguments
///
/// * `config` - AI provider configuration including provider type, model, and API key
/// * `request` - Generation parameters including:
///   - Persona name and description for context
///   - Target granularity level (e.g., "Hair", "Face")
///   - Counts of positive/negative tokens to generate
///   - Existing tokens to avoid duplicates
///   - Current prompt state for budget awareness
///   - Optional custom AI instructions
///
/// # Returns
///
/// `TokenGenerationResponse` containing:
/// - `positive_tokens`: Suggested tokens with weights and rationales
/// - `negative_tokens`: Suggested exclusion tokens
/// - Provider and model used for attribution
///
/// # Errors
///
/// Returns `AppError::Internal` if the AI request fails or response parsing fails.
#[tauri::command]
pub async fn generate_ai_token_suggestions(
    config: AiProviderConfig,
    request: TokenGenerationRequest,
) -> Result<TokenGenerationResponse, AppError> {
    ai::generate_tokens(&config, &request).await
}

// ============================================================================
// Provider Configuration
// ============================================================================
//
// Utilities for configuring AI providers.

/// Returns the default configuration for an AI provider.
///
/// Creates a new configuration with the provider's default model and no API key.
/// The frontend uses this as a starting point before adding the user's API key.
///
/// # Arguments
///
/// * `provider` - The AI provider enum variant
///
/// # Returns
///
/// Default `AiProviderConfig` for the specified provider.
#[tauri::command]
#[must_use] 
pub fn get_ai_provider_config(provider: AiProvider) -> AiProviderConfig {
    AiProviderConfig::new(provider)
}

/// Returns metadata for all supported AI providers.
///
/// This is the single source of truth for provider information, ensuring
/// frontend and backend stay synchronized. Includes display names, default
/// models, and API key requirements.
///
/// # Returns
///
/// Vector of `AiProviderMetadata` for all providers:
/// - `id`: Lowercase identifier for serialization
/// - `display_name`: Human-readable name for UI
/// - `requires_api_key`: Whether provider needs authentication
/// - `default_model`: Recommended model for the provider
#[tauri::command]
#[must_use] 
pub fn get_ai_provider_metadata() -> Vec<AiProviderMetadata> {
    AiProvider::all_metadata()
}
