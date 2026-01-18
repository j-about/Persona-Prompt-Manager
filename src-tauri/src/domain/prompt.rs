//! Prompt Composition Logic
//!
//! This module implements the core prompt assembly algorithm that transforms
//! a persona's tokens into ready-to-use positive and negative prompts for
//! image generation tools.
//!
//! # Composition Algorithm
//!
//! The `PromptComposer` processes tokens through these stages:
//!
//! 1. **Granularity Selection**: Filter to specified levels or use all
//! 2. **Ordering**: Sort levels by `display_order`, tokens within levels
//! 3. **Polarity Separation**: Route tokens to positive or negative output
//! 4. **Weight Formatting**: Apply `(token:weight)` syntax if enabled
//! 5. **Ad-hoc Injection**: Insert additional tokens at beginning or end
//! 6. **Assembly**: Join with separator and create breakdown
//!
//! # Output Format
//!
//! The composed prompt follows Stable Diffusion conventions:
//! - Tokens joined by commas: `token1, token2, token3`
//! - Weighted tokens: `(emphasized token:1.2)`
//! - Separate positive and negative prompt strings

use serde::{Deserialize, Serialize};

use super::token::{GranularityLevel, Token, TokenPolarity};

/// The final assembled prompt ready for image generation.
///
/// Contains both positive and negative prompts along with metadata
/// useful for UI display and token budget tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposedPrompt {
    /// The positive prompt string (desired characteristics)
    pub positive_prompt: String,
    /// The negative prompt string (undesired characteristics)
    pub negative_prompt: String,
    /// Count of positive token parts (including ad-hoc)
    pub positive_token_count: usize,
    /// Count of negative token parts (including ad-hoc)
    pub negative_token_count: usize,
    /// Detailed breakdown by granularity level
    pub breakdown: PromptBreakdown,
}

/// Breakdown showing which tokens contributed from each granularity level.
///
/// Used by the UI to display how the prompt was assembled and which
/// sections contributed to each part.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptBreakdown {
    /// Sections in composition order
    pub sections: Vec<GranularitySection>,
}

/// Tokens from a single granularity level, separated by polarity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GranularitySection {
    /// Granularity level ID (e.g., "hair", "face")
    pub granularity_id: String,
    /// Human-readable level name
    pub granularity_name: String,
    /// Positive token contents (without weight formatting)
    pub positive_tokens: Vec<String>,
    /// Negative token contents (without weight formatting)
    pub negative_tokens: Vec<String>,
}

/// Configuration options for prompt composition.
///
/// All fields have sensible defaults via `Default` implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionOptions {
    /// Whether to apply weight formatting to tokens (default: true)
    #[serde(default = "default_prompt_include_weights")]
    pub include_weights: bool,
    /// String used to join tokens (default: ", ")
    #[serde(default = "default_prompt_token_separator")]
    pub separator: String,
    /// Granularity level IDs to include, in order (default: all levels)
    #[serde(default)]
    pub granularity_ids: Vec<String>,
    /// Additional positive tokens to inject
    #[serde(default)]
    pub adhoc_positive: Option<String>,
    /// Additional negative tokens to inject
    #[serde(default)]
    pub adhoc_negative: Option<String>,
    /// Placement of ad-hoc tokens (default: End)
    #[serde(default)]
    pub adhoc_position: AdhocPosition,
}

const fn default_prompt_include_weights() -> bool {
    true
}

fn default_prompt_token_separator() -> String {
    ", ".to_string()
}

/// Determines where ad-hoc tokens are inserted in the composed prompt.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AdhocPosition {
    /// Insert before all persona tokens
    Beginning,
    /// Insert after all persona tokens
    #[default]
    End,
}

/// Request payload for prompt composition via IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposePromptRequest {
    /// Target persona UUID
    pub persona_id: String,
    /// Composition options
    #[serde(default)]
    pub options: CompositionOptions,
}

impl Default for CompositionOptions {
    fn default() -> Self {
        Self {
            include_weights: true,
            separator: ", ".to_string(),
            granularity_ids: vec![],
            adhoc_positive: None,
            adhoc_negative: None,
            adhoc_position: AdhocPosition::End,
        }
    }
}

/// Stateless prompt composition service.
///
/// Assembles tokens into prompt strings following image generation conventions.
pub struct PromptComposer;

impl PromptComposer {
    /// Composes a prompt from tokens according to the specified options.
    ///
    /// # Arguments
    ///
    /// * `tokens` - All tokens for the persona
    /// * `granularity_levels` - Available granularity level definitions
    /// * `options` - Composition configuration
    ///
    /// # Algorithm
    ///
    /// 1. Determine which granularity levels to include
    /// 2. Optionally inject ad-hoc tokens at the beginning
    /// 3. For each granularity level (in order):
    ///    - Sort tokens by `display_order`
    ///    - Format each token (apply weight if configured)
    ///    - Add to positive or negative parts based on polarity
    /// 4. Optionally inject ad-hoc tokens at the end
    /// 5. Join parts with separator
    #[must_use] 
    pub fn compose(
        tokens: &[Token],
        granularity_levels: &[GranularityLevel],
        options: &CompositionOptions,
    ) -> ComposedPrompt {
        let mut positive_parts: Vec<String> = Vec::new();
        let mut negative_parts: Vec<String> = Vec::new();
        let mut sections: Vec<GranularitySection> = Vec::new();

        // Determine which granularities to include and their order
        let granularities_to_use: Vec<&GranularityLevel> = if options.granularity_ids.is_empty() {
            let mut levels: Vec<_> = granularity_levels.iter().collect();
            levels.sort_by_key(|l| l.display_order);
            levels
        } else {
            options
                .granularity_ids
                .iter()
                .filter_map(|id| granularity_levels.iter().find(|l| &l.id == id))
                .collect()
        };

        // Inject ad-hoc tokens at beginning if configured
        if options.adhoc_position == AdhocPosition::Beginning {
            if let Some(adhoc) = &options.adhoc_positive {
                if !adhoc.trim().is_empty() {
                    positive_parts.push(adhoc.trim().to_string());
                }
            }
            if let Some(adhoc) = &options.adhoc_negative {
                if !adhoc.trim().is_empty() {
                    negative_parts.push(adhoc.trim().to_string());
                }
            }
        }

        // Process each granularity level
        for level in &granularities_to_use {
            let level_tokens: Vec<&Token> = tokens
                .iter()
                .filter(|t| t.granularity_id == level.id)
                .collect();

            if level_tokens.is_empty() {
                continue;
            }

            let mut section_positive: Vec<String> = Vec::new();
            let mut section_negative: Vec<String> = Vec::new();

            // Sort tokens by display order within the level
            let mut sorted_tokens = level_tokens.clone();
            sorted_tokens.sort_by_key(|t| t.display_order);

            for token in sorted_tokens {
                let formatted = token.format_for_prompt(options.include_weights);

                match token.polarity {
                    TokenPolarity::Positive => {
                        positive_parts.push(formatted.clone());
                        section_positive.push(token.content.clone());
                    }
                    TokenPolarity::Negative => {
                        negative_parts.push(formatted.clone());
                        section_negative.push(token.content.clone());
                    }
                }
            }

            if !section_positive.is_empty() || !section_negative.is_empty() {
                sections.push(GranularitySection {
                    granularity_id: level.id.clone(),
                    granularity_name: level.name.clone(),
                    positive_tokens: section_positive,
                    negative_tokens: section_negative,
                });
            }
        }

        // Inject ad-hoc tokens at end if configured
        if options.adhoc_position == AdhocPosition::End {
            if let Some(adhoc) = &options.adhoc_positive {
                if !adhoc.trim().is_empty() {
                    positive_parts.push(adhoc.trim().to_string());
                }
            }
            if let Some(adhoc) = &options.adhoc_negative {
                if !adhoc.trim().is_empty() {
                    negative_parts.push(adhoc.trim().to_string());
                }
            }
        }

        ComposedPrompt {
            positive_prompt: positive_parts.join(&options.separator),
            negative_prompt: negative_parts.join(&options.separator),
            positive_token_count: positive_parts.len(),
            negative_token_count: negative_parts.len(),
            breakdown: PromptBreakdown { sections },
        }
    }
}
