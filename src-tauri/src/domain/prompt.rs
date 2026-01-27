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
//! 2. **Ordering**: Sort by global `display_order` (user-defined sequence)
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
    /// DaisyUI color name for styling (e.g., "primary", "accent")
    pub granularity_color: String,
    /// Positive token contents (with weight formatting if enabled)
    pub positive_tokens: Vec<String>,
    /// Negative token contents (with weight formatting if enabled)
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
    /// 1. Filter tokens by selected granularity levels (or use all)
    /// 2. Sort tokens by global `display_order` (user-defined sequence)
    /// 3. Optionally inject ad-hoc tokens at the beginning
    /// 4. Process each token in order:
    ///    - Format token (apply weight if configured)
    ///    - Add to positive or negative parts based on polarity
    ///    - Track breakdown by granularity for UI display
    /// 5. Optionally inject ad-hoc tokens at the end
    /// 6. Join parts with separator
    #[must_use]
    pub fn compose(
        tokens: &[Token],
        granularity_levels: &[GranularityLevel],
        options: &CompositionOptions,
    ) -> ComposedPrompt {
        use std::collections::HashMap;

        let mut positive_parts: Vec<String> = Vec::new();
        let mut negative_parts: Vec<String> = Vec::new();

        // Determine which granularities to include
        let allowed_granularities: Option<std::collections::HashSet<&str>> =
            if options.granularity_ids.is_empty() {
                None // All granularities allowed
            } else {
                Some(
                    options
                        .granularity_ids
                        .iter()
                        .map(|s| s.as_str())
                        .collect(),
                )
            };

        // Filter and sort tokens by global display_order
        let mut sorted_tokens: Vec<&Token> = tokens
            .iter()
            .filter(|t| {
                allowed_granularities
                    .as_ref()
                    .map_or(true, |allowed| allowed.contains(t.granularity_id.as_str()))
            })
            .collect();
        sorted_tokens.sort_by_key(|t| t.display_order);

        // Track breakdown by granularity (for informational purposes)
        let mut section_map: HashMap<String, GranularitySection> = HashMap::new();

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

        // Process tokens in user-defined order
        for token in sorted_tokens {
            let formatted = token.format_for_prompt(options.include_weights);

            match token.polarity {
                TokenPolarity::Positive => {
                    positive_parts.push(formatted.clone());
                }
                TokenPolarity::Negative => {
                    negative_parts.push(formatted.clone());
                }
            }

            // Track breakdown by granularity
            let section = section_map
                .entry(token.granularity_id.clone())
                .or_insert_with(|| {
                    let level = granularity_levels
                        .iter()
                        .find(|l| l.id == token.granularity_id);
                    GranularitySection {
                        granularity_id: token.granularity_id.clone(),
                        granularity_name: level
                            .map_or_else(|| "Unknown".to_string(), |l| l.name.clone()),
                        granularity_color: level
                            .map_or_else(|| "base".to_string(), |l| l.color.clone()),
                        positive_tokens: Vec::new(),
                        negative_tokens: Vec::new(),
                    }
                });

            match token.polarity {
                TokenPolarity::Positive => section.positive_tokens.push(formatted.clone()),
                TokenPolarity::Negative => section.negative_tokens.push(formatted.clone()),
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

        // Convert section_map to ordered vector (by granularity display_order for breakdown)
        let mut sections: Vec<GranularitySection> = granularity_levels
            .iter()
            .filter_map(|l| section_map.remove(&l.id))
            .collect();
        // Add any remaining sections (unknown granularities) at the end
        sections.extend(section_map.into_values());

        ComposedPrompt {
            positive_prompt: positive_parts.join(&options.separator),
            negative_prompt: negative_parts.join(&options.separator),
            positive_token_count: positive_parts.len(),
            negative_token_count: negative_parts.len(),
            breakdown: PromptBreakdown { sections },
        }
    }
}
