//! Token Domain Entity
//!
//! This module defines tokens, the atomic units of image generation prompts.
//! Tokens are organized hierarchically by granularity level and separated by
//! polarity (positive vs negative).
//!
//! # Token Structure
//!
//! Each token has:
//! - **Content**: The descriptive text (e.g., "red hair", "detailed eyes")
//! - **Weight**: Relative emphasis (1.0 = normal, >1.0 = more emphasis)
//! - **Polarity**: Whether it's desired (positive) or undesired (negative)
//! - **Granularity**: Which body/style category it belongs to
//!
//! # Granularity Levels
//!
//! Tokens are organized into seven hierarchical levels:
//! 1. **Style**: Overall artistic style (e.g., "masterpiece", "anime")
//! 2. **General**: General physical traits (e.g., "pale skin", "tan complexion")
//! 3. **Hair**: Hair-related tokens (e.g., "red hair", "long hair")
//! 4. **Face**: Facial features (e.g., "blue eyes", "freckles")
//! 5. **Upper Body**: Torso, chest, arms, shoulders (e.g., "muscular arms", "broad shoulders")
//! 6. **Midsection**: Waist, hips, midriff (e.g., "narrow waist", "wide hips")
//! 7. **Lower Body**: Legs, thighs, feet (e.g., "long legs", "slender ankles")

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Token polarity determines whether a token describes desired or undesired characteristics.
///
/// - **Positive**: Include this characteristic in the generated image
/// - **Negative**: Exclude this characteristic from the generated image
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenPolarity {
    /// Token describes a desired characteristic
    Positive,
    /// Token describes an undesired characteristic (goes in negative prompt)
    Negative,
}

impl TokenPolarity {
    /// Returns the lowercase string representation for database storage.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Negative => "negative",
        }
    }

    /// Parses from database string representation.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "positive" => Some(Self::Positive),
            "negative" => Some(Self::Negative),
            _ => None,
        }
    }
}

/// Enumeration of the seven granularity levels for token organization.
///
/// These levels represent a hierarchical breakdown of character attributes,
/// enabling selective prompt composition where users can choose which
/// aspects of a persona to include.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Granularity {
    /// Overall artistic style and quality tags
    Style,
    /// General physical traits (skin tone, complexion)
    General,
    /// Hair color, length, style
    Hair,
    /// Eyes, face shape, facial features
    Face,
    /// Torso, chest, arms, shoulders
    UpperBody,
    /// Waist, hips, midriff
    Midsection,
    /// Legs, thighs, feet
    LowerBody,
}

impl Granularity {
    /// Returns the `snake_case` string ID used in database and serialization.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Style => "style",
            Self::General => "general",
            Self::Hair => "hair",
            Self::Face => "face",
            Self::UpperBody => "upper_body",
            Self::Midsection => "midsection",
            Self::LowerBody => "lower_body",
        }
    }

    /// Parses from string representation.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "style" => Some(Self::Style),
            "general" => Some(Self::General),
            "hair" => Some(Self::Hair),
            "face" => Some(Self::Face),
            "upper_body" => Some(Self::UpperBody),
            "midsection" => Some(Self::Midsection),
            "lower_body" => Some(Self::LowerBody),
            _ => None,
        }
    }

    /// Returns the human-readable display name for UI.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Style => "Style",
            Self::General => "General",
            Self::Hair => "Hair",
            Self::Face => "Face",
            Self::UpperBody => "Upper Body",
            Self::Midsection => "Midsection",
            Self::LowerBody => "Lower Body",
        }
    }

    /// Returns the sort order for display (0 = first, 6 = last).
    #[must_use]
    pub const fn display_order(&self) -> i32 {
        match self {
            Self::Style => 0,
            Self::General => 1,
            Self::Hair => 2,
            Self::Face => 3,
            Self::UpperBody => 4,
            Self::Midsection => 5,
            Self::LowerBody => 6,
        }
    }

    /// Returns all granularity variants in display order.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Style,
            Self::General,
            Self::Hair,
            Self::Face,
            Self::UpperBody,
            Self::Midsection,
            Self::LowerBody,
        ]
    }
}

/// Serializable granularity level for frontend communication.
///
/// This struct converts the `Granularity` enum into a frontend-friendly format
/// with explicit `id`, `name`, and `display_order` fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GranularityLevel {
    /// Unique identifier (matches `Granularity::as_str()`)
    pub id: String,
    /// Human-readable display name
    pub name: String,
    /// Sort order for UI presentation
    pub display_order: i32,
    /// Whether this is a built-in level (always true currently)
    pub is_default: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// A token represents a single descriptive element within a prompt.
///
/// Tokens are the atomic building blocks of prompts. They are organized
/// by persona, granularity level, and polarity, with individual weights
/// for fine-grained control over emphasis.
///
/// # Weight Formatting
///
/// When composed into prompts, tokens with non-default weights are formatted as:
/// - Weight 1.0: `content` (no modification)
/// - Weight != 1.0: `(content:weight)` (e.g., "(red hair:1.2)")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// Unique identifier (UUID v4)
    pub id: String,
    /// Parent persona UUID (foreign key)
    pub persona_id: String,
    /// Granularity level ID (e.g., "hair", "face")
    pub granularity_id: String,
    /// Whether this is a positive or negative token
    pub polarity: TokenPolarity,
    /// The actual descriptive text
    pub content: String,
    /// Weight modifier (1.0 = normal, >1 = more emphasis, <1 = less)
    pub weight: f64,
    /// Sort order within granularity/polarity group
    pub display_order: i32,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modification timestamp
    pub updated_at: DateTime<Utc>,
}

/// Request payload for creating a single token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTokenRequest {
    /// Parent persona UUID
    pub persona_id: String,
    /// Granularity level ID
    pub granularity_id: String,
    /// Token polarity
    pub polarity: TokenPolarity,
    /// Descriptive content
    pub content: String,
    /// Weight modifier (defaults to 1.0)
    #[serde(default = "default_weight")]
    pub weight: f64,
}

const fn default_weight() -> f64 {
    1.0
}

/// Request payload for creating multiple tokens from comma-separated input.
///
/// This is the primary method for bulk token entry. The `contents` field
/// is split on commas, with each trimmed value becoming a separate token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCreateTokenRequest {
    /// Parent persona UUID
    pub persona_id: String,
    /// Granularity level ID for all created tokens
    pub granularity_id: String,
    /// Polarity for all created tokens
    pub polarity: TokenPolarity,
    /// Comma-separated token contents (e.g., "red hair, long hair, flowing")
    pub contents: String,
    /// Weight modifier applied to all created tokens
    #[serde(default = "default_weight")]
    pub weight: f64,
}

/// Request payload for updating an existing token.
///
/// All fields are optional; only provided fields are updated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTokenRequest {
    /// New content text
    pub content: Option<String>,
    /// New weight value
    pub weight: Option<f64>,
    /// New granularity level
    pub granularity_id: Option<String>,
    /// New polarity
    pub polarity: Option<TokenPolarity>,
}

impl From<Granularity> for GranularityLevel {
    fn from(g: Granularity) -> Self {
        Self {
            id: g.as_str().to_string(),
            name: g.display_name().to_string(),
            display_order: g.display_order(),
            is_default: true,
            created_at: Utc::now(),
        }
    }
}

impl GranularityLevel {
    /// Returns all granularity levels in display order.
    #[must_use]
    pub fn all() -> Vec<Self> {
        Granularity::all().iter().map(|&g| g.into()).collect()
    }
}

impl Token {
    /// Creates a new token with auto-generated UUID and current timestamps.
    ///
    /// # Arguments
    ///
    /// * `persona_id` - Parent persona UUID
    /// * `granularity_id` - Granularity level ID
    /// * `polarity` - Positive or negative
    /// * `content` - Descriptive text
    /// * `weight` - Weight modifier
    /// * `display_order` - Sort position within group
    #[must_use]
    pub fn new(
        persona_id: String,
        granularity_id: String,
        polarity: TokenPolarity,
        content: String,
        weight: f64,
        display_order: i32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            persona_id,
            granularity_id,
            polarity,
            content,
            weight,
            display_order,
            created_at: now,
            updated_at: now,
        }
    }

    /// Applies partial updates from a request, refreshing `updated_at`.
    pub fn update(&mut self, request: &UpdateTokenRequest) {
        if let Some(content) = &request.content {
            self.content = content.clone();
        }
        if let Some(weight) = request.weight {
            self.weight = weight;
        }
        if let Some(granularity_id) = &request.granularity_id {
            self.granularity_id = granularity_id.clone();
        }
        if let Some(polarity) = request.polarity {
            self.polarity = polarity;
        }
        self.updated_at = Utc::now();
    }

    /// Formats the token for inclusion in a prompt string.
    ///
    /// # Arguments
    ///
    /// * `include_weight` - Whether to add weight modifiers
    ///
    /// # Returns
    ///
    /// - If `include_weight` is false or weight is 1.0: returns content as-is
    /// - Otherwise: returns `(content:weight)` format
    #[must_use]
    pub fn format_for_prompt(&self, include_weight: bool) -> String {
        if include_weight && (self.weight - 1.0).abs() > f64::EPSILON {
            format!("({}:{:.1})", self.content, self.weight)
        } else {
            self.content.clone()
        }
    }
}

impl BatchCreateTokenRequest {
    /// Parses the comma-separated contents into individual token strings.
    ///
    /// Empty strings after trimming are filtered out.
    #[must_use]
    pub fn parse_contents(&self) -> Vec<String> {
        self.contents
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}
