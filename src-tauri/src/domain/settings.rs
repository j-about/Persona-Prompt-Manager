//! Application Settings Domain Types
//!
//! This module defines types for application-wide configuration and user preferences.
//! Settings are persisted and applied globally across all personas and sessions.
//!
//! # Available Settings
//!
//! - **Theme**: UI appearance (light, dark, or system)
//! - **Token Separator**: Character(s) used between tokens in prompts
//! - **Include Weights**: Whether to format tokens with weight modifiers
//! - **Default Max Tokens**: Token limit for new personas

use serde::{Deserialize, Serialize};

/// Application theme preference.
///
/// Controls the UI color scheme. The `System` option follows the OS setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    /// Light color scheme
    Light,
    /// Dark color scheme
    Dark,
    /// Follow operating system preference
    #[default]
    System,
}

/// Complete application settings container.
///
/// These settings affect prompt composition and UI behavior globally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// UI color theme
    pub theme: Theme,
    /// Separator string used when joining tokens (default: ", ")
    pub token_separator: String,
    /// Whether to include weight modifiers in composed prompts
    pub include_weights: bool,
    /// Default token limit for new persona generation params
    pub default_max_tokens: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            token_separator: ", ".to_string(),
            include_weights: true,
            default_max_tokens: 77,
        }
    }
}

/// Keys for individual settings in key-value storage.
///
/// Used for database storage and retrieval of individual settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingKey {
    /// UI theme preference
    Theme,
    /// Token separator string
    TokenSeparator,
    /// Weight inclusion toggle
    IncludeWeights,
    /// Default token limit
    DefaultMaxTokens,
}

impl SettingKey {
    /// Returns the string key used for database storage.
    #[must_use] 
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Theme => "theme",
            Self::TokenSeparator => "token_separator",
            Self::IncludeWeights => "include_weights",
            Self::DefaultMaxTokens => "default_max_tokens",
        }
    }

    /// Parses a string key into a `SettingKey` variant.
    #[must_use] 
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "theme" => Some(Self::Theme),
            "token_separator" => Some(Self::TokenSeparator),
            "include_weights" => Some(Self::IncludeWeights),
            "default_max_tokens" => Some(Self::DefaultMaxTokens),
            _ => None,
        }
    }
}
