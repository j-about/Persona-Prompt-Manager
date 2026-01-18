//! Persona Domain Entity
//!
//! This module defines the core `Persona` entity and related types. A persona represents
//! a complete character profile for AI image generation, serving as the primary
//! organizational unit for tokens and generation settings.
//!
//! # Persona Composition
//!
//! Each persona aggregates:
//! - **Identity**: Name, description, and organizational tags
//! - **Tokens**: Descriptive elements organized by granularity (stored separately)
//! - **Generation Params**: Image generation settings (model, seed, steps, etc.)
//! - **AI Configuration**: Optional LLM provider settings for token generation

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;
use uuid::Uuid;

use super::DEFAULT_IMAGE_MODEL_ID;

/// A Persona represents a complete fictional character profile for AI image generation.
///
/// Personas serve as the top-level organizational unit, containing metadata and
/// configuration while referencing associated tokens and generation parameters
/// via their `id` field.
///
/// # Fields
///
/// - `id`: UUID string, generated on creation
/// - `name`: Unique display name for identification
/// - `description`: Optional long-form character description
/// - `tags`: Organizational labels for filtering and grouping
/// - `ai_*`: Optional configuration for AI-powered token generation
/// - `created_at`/`updated_at`: Timestamps for auditing and sorting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    /// Unique identifier (UUID v4)
    pub id: String,
    /// Display name, must be unique across all personas
    pub name: String,
    /// Optional detailed description of the character
    pub description: Option<String>,
    /// Organizational tags for filtering (e.g., "fantasy", "anime")
    pub tags: Vec<String>,
    /// AI provider ID for token generation (e.g., "openai", "anthropic")
    pub ai_provider_id: Option<String>,
    /// AI model ID for token generation (e.g., "gpt-4o-mini")
    pub ai_model_id: Option<String>,
    /// Custom instructions passed to AI during token generation
    pub ai_instructions: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modification timestamp
    pub updated_at: DateTime<Utc>,
}

/// Image generation parameters associated with a persona.
///
/// These settings correspond to typical Stable Diffusion / SDXL / FLUX parameters
/// and are stored alongside the persona for reproducible generations.
///
/// # Default Values
///
/// - `model_id`: See [`DEFAULT_IMAGE_MODEL_ID`]
/// - `seed`: -1 (random)
/// - `steps`: 30
/// - `cfg_scale`: 7.0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationParams {
    /// UUID of the parent persona (foreign key)
    pub persona_id: String,
    /// Image generation model identifier, used for tokenizer selection
    pub model_id: String,
    /// Random seed for reproducibility (-1 for random)
    pub seed: i64,
    /// Number of diffusion steps
    pub steps: u32,
    /// Classifier-free guidance scale
    pub cfg_scale: f32,
    /// Sampler algorithm (e.g., "euler", "dpm++")
    pub sampler: Option<String>,
    /// Scheduler algorithm (e.g., "karras", "exponential", "normal")
    pub scheduler: Option<String>,
}

/// Request payload for creating a new persona.
///
/// Only the `name` field is required; description and tags default to empty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePersonaRequest {
    /// Unique name for the persona (required)
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Optional tags (defaults to empty vector)
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Request payload for updating an existing persona.
///
/// All fields are optional; only provided fields are updated.
/// Omitted fields retain their current values.
///
/// For AI fields (`ai_provider_id`, `ai_model_id`, `ai_instructions`), the double
/// option pattern is used to distinguish between:
/// - `None`: Field not provided in JSON, retain current value
/// - `Some(None)`: Field explicitly set to `null` in JSON, clear the value
/// - `Some(Some(value))`: Field has a value in JSON, update to that value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePersonaRequest {
    /// New name (must be unique if provided)
    pub name: Option<String>,
    /// New description
    pub description: Option<String>,
    /// New tags
    pub tags: Option<Vec<String>>,
    /// New AI provider ID: None = not provided, Some(None) = clear, Some(Some(id)) = set
    #[serde(default, with = "double_option")]
    pub ai_provider_id: Option<Option<String>>,
    /// New AI model ID: None = not provided, Some(None) = clear, Some(Some(id)) = set
    #[serde(default, with = "double_option")]
    pub ai_model_id: Option<Option<String>>,
    /// New AI instructions: None = not provided, Some(None) = clear, Some(Some(text)) = set
    #[serde(default, with = "double_option")]
    pub ai_instructions: Option<Option<String>>,
}

impl Persona {
    /// Creates a new persona with auto-generated UUID and current timestamps.
    ///
    /// AI configuration fields are initialized to `None`.
    ///
    /// # Arguments
    ///
    /// * `name` - Display name for the persona
    /// * `description` - Optional character description
    /// * `tags` - Organizational tags
    #[must_use] 
    pub fn new(name: String, description: Option<String>, tags: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            tags,
            ai_provider_id: None,
            ai_model_id: None,
            ai_instructions: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Applies partial updates from a request, refreshing `updated_at`.
    ///
    /// Only fields present in the request are modified. For AI fields using the
    /// double option pattern, `Some(None)` clears the field while `None` leaves
    /// the current value unchanged.
    pub fn update(&mut self, request: &UpdatePersonaRequest) {
        if let Some(name) = &request.name {
            self.name = name.clone();
        }
        if let Some(description) = &request.description {
            self.description = Some(description.clone());
        }
        if let Some(tags) = &request.tags {
            self.tags = tags.clone();
        }
        // AI fields use double option: Some(None) clears, Some(Some(v)) sets, None = no change
        if let Some(ai_provider_id) = &request.ai_provider_id {
            self.ai_provider_id = ai_provider_id.clone();
        }
        if let Some(ai_model_id) = &request.ai_model_id {
            self.ai_model_id = ai_model_id.clone();
        }
        if let Some(ai_instructions) = &request.ai_instructions {
            self.ai_instructions = ai_instructions.clone();
        }
        self.updated_at = Utc::now();
    }
}

impl GenerationParams {
    /// Creates default generation parameters linked to a specific persona.
    #[must_use] 
    pub fn default_for_persona(persona_id: &str) -> Self {
        Self {
            persona_id: persona_id.to_string(),
            ..Default::default()
        }
    }
}

impl Default for GenerationParams {
    fn default() -> Self {
        Self {
            persona_id: String::new(),
            model_id: DEFAULT_IMAGE_MODEL_ID.to_string(),
            seed: -1,
            steps: 30,
            cfg_scale: 7.0,
            sampler: None,
            scheduler: None,
        }
    }
}
