//! AI Provider Domain Types
//!
//! This module defines the configuration and request/response types for
//! AI-powered generation using large language models.
//!
//! # Supported Providers
//!
//! The application supports five AI providers:
//!
//! | Provider  | Default Model           | API Key Required |
//! |-----------|-------------------------|------------------|
//! | `OpenAI`    | gpt-5.2-pro             | Yes              |
//! | Anthropic | claude-opus-4-5         | Yes              |
//! | Google    | gemini-3-pro-preview    | Yes              |
//! | xAI       | grok-4-1-fast-reasoning | Yes              |
//! | Ollama    | llama3.2                | No (local)       |
//!
//! # Design Philosophy
//!
//! The Rust backend is the single source of truth for provider metadata.
//! The frontend fetches this information via `get_ai_provider_metadata()`,
//! ensuring consistency and making it easy to add new providers.

use serde::{Deserialize, Serialize};

// ============================================================================
// Provider Configuration
// ============================================================================
//
// Core types for AI provider configuration.

/// Enumeration of supported AI providers for token generation.
///
/// Each provider has specific characteristics regarding API access,
/// default models, and authentication requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
    /// `OpenAI` (GPT models)
    OpenAI,
    /// Anthropic (Claude models)
    Anthropic,
    /// Google AI (Gemini models)
    Google,
    /// xAI (Grok models)
    XAi,
    /// Ollama (local LLM runtime)
    Ollama,
}

impl AiProvider {
    /// Returns the human-readable display name for UI presentation.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::OpenAI => "OpenAI",
            Self::Anthropic => "Anthropic",
            Self::Google => "Google AI",
            Self::XAi => "xAI (Grok)",
            Self::Ollama => "Ollama",
        }
    }

    /// Returns whether this provider requires an API key for authentication.
    ///
    /// Ollama runs locally and doesn't require authentication.
    #[must_use]
    pub const fn requires_api_key(&self) -> bool {
        match self {
            Self::OpenAI | Self::Anthropic | Self::Google | Self::XAi => true,
            Self::Ollama => false,
        }
    }

    /// Returns the recommended default model for this provider.
    #[must_use]
    pub const fn default_model(&self) -> &'static str {
        match self {
            Self::OpenAI => "gpt-5.2-pro",
            Self::Anthropic => "claude-opus-4-5",
            Self::Google => "gemini-3-pro-preview",
            Self::XAi => "grok-4-1-fast-reasoning",
            Self::Ollama => "llama3.2",
        }
    }

    /// Returns the default base URL if the provider supports custom endpoints.
    #[must_use]
    pub const fn default_base_url(&self) -> Option<&'static str> {
        None
    }

    /// Returns all available provider variants.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::OpenAI,
            Self::Anthropic,
            Self::Google,
            Self::XAi,
            Self::Ollama,
        ]
    }

    /// Returns the lowercase string identifier used for serialization.
    #[must_use]
    pub const fn id(&self) -> &'static str {
        match self {
            Self::OpenAI => "openai",
            Self::Anthropic => "anthropic",
            Self::Google => "google",
            Self::XAi => "xai",
            Self::Ollama => "ollama",
        }
    }

    /// Creates complete metadata for frontend consumption.
    pub fn metadata(&self) -> AiProviderMetadata {
        AiProviderMetadata {
            id: self.id().to_string(),
            display_name: self.display_name().to_string(),
            requires_api_key: self.requires_api_key(),
            default_model: self.default_model().to_string(),
            default_base_url: self.default_base_url().map(String::from),
        }
    }

    /// Returns metadata for all providers (single source of truth).
    #[must_use]
    pub fn all_metadata() -> Vec<AiProviderMetadata> {
        Self::all().iter().map(Self::metadata).collect()
    }
}

/// Complete provider metadata for frontend synchronization.
///
/// This struct contains all information the frontend needs to display
/// provider options and validate configuration without duplicating logic.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderMetadata {
    /// Lowercase provider identifier for serialization
    pub id: String,
    /// Human-readable name for UI display
    pub display_name: String,
    /// Whether API key configuration is required
    pub requires_api_key: bool,
    /// Recommended model for this provider
    pub default_model: String,
    /// Default API endpoint (if customizable)
    pub default_base_url: Option<String>,
}

/// Configuration for connecting to an AI provider.
///
/// This struct is populated by the frontend and passed to the backend
/// for token generation requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderConfig {
    /// Target provider
    pub provider: AiProvider,
    /// Model to use for generation
    pub model: String,
    /// API key (retrieved from keyring, optional for Ollama)
    pub api_key: Option<String>,
    /// Custom base URL (optional)
    pub base_url: Option<String>,
}

impl AiProviderConfig {
    /// Creates a new configuration with provider defaults.
    pub fn new(provider: AiProvider) -> Self {
        Self {
            model: provider.default_model().to_string(),
            api_key: None,
            base_url: provider.default_base_url().map(String::from),
            provider,
        }
    }
}

/// Runtime status of an AI provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderStatus {
    /// The provider this status describes
    pub provider: AiProvider,
    /// Whether the provider is fully configured
    pub configured: bool,
    /// Whether an API key is stored in the keyring
    pub has_api_key: bool,
    /// Currently selected model (if configured)
    pub model: Option<String>,
    /// Error message if configuration check failed
    pub error: Option<String>,
}

// ============================================================================
// Shared Types
// ============================================================================
//
// Types used by both Persona Generation and Token Generation.

/// A single token suggestion from AI generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedToken {
    /// The suggested token text
    pub content: String,
    /// Recommended weight (1.0 = normal emphasis)
    pub suggested_weight: f64,
    /// AI's explanation for suggesting this token
    pub rationale: Option<String>,
}

// ============================================================================
// Persona Generation Types
// ============================================================================
//
// Types for creating complete persona profiles with tokens organized by body region.

/// General physical traits for persona generation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalCriteriaGeneral {
    /// Skin tone (e.g., "fair", "medium", "dark")
    pub skin_tone: Option<String>,
    /// Body type (e.g., "slim", "athletic", "curvy")
    pub body_type: Option<String>,
    /// Height description (e.g., "short", "average", "tall")
    pub height: Option<String>,
    /// Apparent age range (e.g., "young adult", "mature")
    pub age: Option<String>,
    /// Posture type (e.g., "upright", "relaxed", "slouched")
    pub posture: Option<String>,
    /// Build proportion (e.g., "long-limbed", "proportionate", "compact")
    pub build_proportion: Option<String>,
    /// Skin texture (e.g., "smooth", "poreless", "dewy")
    pub skin_texture: Option<String>,
    /// Distinctive marks (e.g., "freckles", "moles", "dimples")
    pub distinctive_marks: Option<String>,
    /// Complexion (e.g., "clear", "sun-kissed", "rosy")
    pub complexion: Option<String>,
}

/// Hair characteristics for persona generation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalCriteriaHair {
    /// Hair color main category (e.g., "Black", "Brown", "Blonde")
    pub color: Option<String>,
    /// Hair color shade (e.g., "Jet Black", "Chestnut", "Platinum")
    pub color_shade: Option<String>,
    /// Hair length (e.g., "short", "medium", "long")
    pub length: Option<String>,
    /// Hair style (e.g., "straight", "wavy", "braided")
    pub style: Option<String>,
    /// Hair texture (e.g., "fine", "thick", "silky")
    pub texture: Option<String>,
}

/// Facial features for persona generation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalCriteriaFace {
    /// Eye color (e.g., "brown", "blue", "green")
    pub eye_color: Option<String>,
    /// Eye shape (e.g., "almond", "round", "hooded")
    pub eye_shape: Option<String>,
    /// Face shape (e.g., "oval", "round", "heart")
    pub face_shape: Option<String>,
    /// Nose shape (e.g., "straight", "aquiline", "button")
    pub nose_shape: Option<String>,
    /// Lip shape (e.g., "thin", "full", "cupid's bow")
    pub lip_shape: Option<String>,
    /// Eyebrow shape (e.g., "arched", "straight", "rounded")
    pub eyebrow_shape: Option<String>,
    /// Chin shape (e.g., "pointed", "square", "round")
    pub chin_shape: Option<String>,
    /// Jawline type (e.g., "square", "sharp", "rounded")
    pub jawline: Option<String>,
    /// Forehead type (e.g., "high", "low", "wide")
    pub forehead: Option<String>,
    /// Cheekbone prominence (e.g., "high", "low", "prominent")
    pub cheekbones: Option<String>,
    /// Teeth appearance (e.g., "straight", "gap-toothed", "perfect")
    pub teeth: Option<String>,
    /// Smile type (e.g., "wide", "subtle", "warm")
    pub smile: Option<String>,
}

/// Upper body characteristics for persona generation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalCriteriaUpperBody {
    /// Upper body build (e.g., "slim", "muscular", "broad")
    pub build: Option<String>,
    /// Shoulder type (e.g., "narrow", "average", "broad")
    pub shoulders: Option<String>,
    /// Neck type (e.g., "short", "average", "long")
    pub neck: Option<String>,
    /// Chest/bust size (e.g., "small", "medium", "large")
    pub chest: Option<String>,
    /// Arm type (e.g., "slender", "toned", "muscular")
    pub arms: Option<String>,
    /// Back type (e.g., "narrow", "broad", "athletic")
    pub back: Option<String>,
    /// Hand type (e.g., "slender", "broad", "delicate")
    pub hands: Option<String>,
    /// Nail type (e.g., "short", "long", "manicured")
    pub nails: Option<String>,
}

/// Midsection characteristics for persona generation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalCriteriaMidsection {
    /// Waist type (e.g., "narrow", "average", "wide")
    pub waist: Option<String>,
    /// Hip type (e.g., "narrow", "average", "curvy")
    pub hips: Option<String>,
}

/// Lower body characteristics for persona generation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalCriteriaLowerBody {
    /// Leg type (e.g., "short", "long", "athletic")
    pub legs: Option<String>,
    /// Lower body build (e.g., "slim", "toned", "curvy")
    pub build: Option<String>,
    /// Feet type (e.g., "small", "average", "large")
    pub feet: Option<String>,
}

/// Physical criteria organized by body region for AI persona generation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalCriteria {
    /// General physical traits
    pub general: Option<PhysicalCriteriaGeneral>,
    /// Hair characteristics
    pub hair: Option<PhysicalCriteriaHair>,
    /// Facial features
    pub face: Option<PhysicalCriteriaFace>,
    /// Upper body characteristics
    pub upper_body: Option<PhysicalCriteriaUpperBody>,
    /// Midsection characteristics
    pub midsection: Option<PhysicalCriteriaMidsection>,
    /// Lower body characteristics
    pub lower_body: Option<PhysicalCriteriaLowerBody>,
}

/// Request payload for AI-based persona generation.
///
/// Contains all inputs needed to generate a complete persona with tokens
/// organized by granularity level.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiPersonaGenerationRequest {
    /// Persona name (required)
    pub name: String,
    /// Desired visual style (e.g., "realistic", "anime", "manga")
    pub style: String,
    /// Character description including age, background, biography
    pub character_description: String,
    /// Physical criteria organized by body region (optional)
    #[serde(default)]
    pub physical_criteria: PhysicalCriteria,
    /// Custom instructions for the AI (optional)
    pub ai_instructions: Option<String>,
    /// Target image model for tokenizer awareness (optional)
    #[serde(default)]
    pub image_model_id: Option<String>,
    /// Existing tags from other personas (for AI to prefer over new ones)
    #[serde(default)]
    pub existing_tags: Vec<String>,
}

/// Generated tokens organized by granularity level.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GeneratedTokensByGranularity {
    /// Style tokens (e.g., "masterpiece", "photorealistic", "anime style", "oil painting")
    pub style: Vec<GeneratedToken>,
    /// General physical trait tokens
    pub general: Vec<GeneratedToken>,
    /// Hair-related tokens
    pub hair: Vec<GeneratedToken>,
    /// Face-related tokens
    pub face: Vec<GeneratedToken>,
    /// Upper body tokens
    pub upper_body: Vec<GeneratedToken>,
    /// Midsection tokens
    pub midsection: Vec<GeneratedToken>,
    /// Lower body tokens
    pub lower_body: Vec<GeneratedToken>,
}

/// Response from AI persona generation.
///
/// Contains the elaborated persona information and generated tokens
/// ready to be saved to the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiPersonaGenerationResponse {
    /// Elaborated persona description based on user input
    pub description: String,
    /// Inferred tags from style and character description
    pub tags: Vec<String>,
    /// Generated tokens organized by granularity
    pub tokens: GeneratedTokensByGranularity,
    /// Provider that handled the request
    pub provider: AiProvider,
    /// Model used for generation
    pub model: String,
}

// ============================================================================
// Token Generation Types
// ============================================================================
//
// Types for generating additional positive/negative tokens during prompt composition.

/// Request payload for AI token generation.
///
/// Contains all context needed for the AI to generate relevant tokens,
/// including persona information, existing tokens, and prompt state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenGenerationRequest {
    /// Persona name for context
    pub persona_name: String,
    /// Optional detailed persona description
    pub persona_description: Option<String>,
    /// Target granularity level (e.g., "Hair", "Face")
    pub granularity_name: String,
    /// Number of positive tokens to generate
    pub positive_count: usize,
    /// Number of negative tokens to generate
    pub negative_count: usize,
    /// Existing positive tokens to avoid duplicates
    pub existing_positive_tokens: Vec<String>,
    /// Existing negative tokens to avoid duplicates
    pub existing_negative_tokens: Vec<String>,
    /// Optional style guidance (e.g., "anime", "realistic")
    pub style_hints: Option<String>,
    /// Target image model for tokenizer awareness
    #[serde(default)]
    pub image_model_id: Option<String>,
    /// Custom instructions to include in the AI prompt
    #[serde(default)]
    pub ai_instructions: Option<String>,
    /// Current positive prompt (for token budget awareness)
    #[serde(default)]
    pub current_positive_prompt: Option<String>,
    /// Current negative prompt (for token budget awareness)
    #[serde(default)]
    pub current_negative_prompt: Option<String>,
    /// Current positive prompt token count
    #[serde(default)]
    pub positive_token_count: Option<usize>,
    /// Current negative prompt token count
    #[serde(default)]
    pub negative_token_count: Option<usize>,
    /// Maximum tokens allowed for the target model
    #[serde(default)]
    pub max_usable_tokens: Option<usize>,
}

/// Response from AI token generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenGenerationResponse {
    /// Generated positive token suggestions
    pub positive_tokens: Vec<GeneratedToken>,
    /// Generated negative token suggestions
    pub negative_tokens: Vec<GeneratedToken>,
    /// Provider that handled the request
    pub provider: AiProvider,
    /// Model used for generation
    pub model: String,
}
