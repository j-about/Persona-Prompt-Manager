//! Export/Import Domain Types
//!
//! This module defines the data structures for persona export and import,
//! enabling backup, sharing, and migration of persona configurations.
//!
//! # Export Format
//!
//! The export format is a versioned JSON structure designed for:
//! - **Compatibility**: Version field enables future format migrations
//! - **Completeness**: Includes all data needed to fully recreate personas
//! - **Portability**: Self-contained with no external dependencies
//!
//! # Supported Operations
//!
//! - **Single Export**: One persona with all tokens and settings
//! - **Bulk Export**: Multiple personas in a single file
//! - **Conflict Handling**: Skip, rename, or replace existing personas

use serde::{Deserialize, Serialize};

use super::persona::{GenerationParams, Persona};
use super::token::{GranularityLevel, Token};

/// Complete export of a single persona with all associated data.
///
/// This format captures everything needed to recreate the persona,
/// including tokens, generation parameters, and granularity definitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaExport {
    /// Export format version for compatibility checking
    pub version: String,
    /// ISO 8601 timestamp when the export was created
    pub exported_at: String,
    /// The persona entity
    pub persona: Persona,
    /// Image generation parameters
    pub generation_params: GenerationParams,
    /// All tokens belonging to this persona
    pub tokens: Vec<Token>,
    /// Granularity level definitions (for validation during import)
    pub granularity_levels: Vec<GranularityLevel>,
}

impl PersonaExport {
    /// Current export format version.
    pub const CURRENT_VERSION: &'static str = "1.0";

    /// Creates a new export for a persona with all its data.
    #[must_use] 
    pub fn new(
        persona: Persona,
        generation_params: GenerationParams,
        tokens: Vec<Token>,
        granularity_levels: Vec<GranularityLevel>,
    ) -> Self {
        Self {
            version: Self::CURRENT_VERSION.to_string(),
            exported_at: chrono::Utc::now().to_rfc3339(),
            persona,
            generation_params,
            tokens,
            granularity_levels,
        }
    }
}

/// Bulk export containing multiple personas.
///
/// Used for full database backup or sharing collections of related personas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkExport {
    /// Export format version
    pub version: String,
    /// ISO 8601 timestamp when the export was created
    pub exported_at: String,
    /// Application identifier for origin tracking
    pub app: String,
    /// All persona exports
    pub personas: Vec<PersonaExport>,
}

impl BulkExport {
    /// Current bulk export format version.
    pub const CURRENT_VERSION: &'static str = "1.0";
    /// Application identifier included in exports.
    pub const APP_NAME: &'static str = "Persona Prompt Manager";

    /// Creates a new bulk export from a list of persona exports.
    #[must_use] 
    pub fn new(personas: Vec<PersonaExport>) -> Self {
        Self {
            version: Self::CURRENT_VERSION.to_string(),
            exported_at: chrono::Utc::now().to_rfc3339(),
            app: Self::APP_NAME.to_string(),
            personas,
        }
    }
}

/// Result of importing a single persona.
///
/// Provides detailed feedback about what was imported, including
/// success/failure status, token counts, and any warnings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    /// Whether the import completed successfully
    pub success: bool,
    /// The imported persona (present only on success)
    pub persona: Option<Persona>,
    /// Number of tokens that were imported
    pub tokens_imported: usize,
    /// Non-fatal issues encountered during import
    pub warnings: Vec<String>,
    /// Error message (present only on failure)
    pub error: Option<String>,
}

impl ImportResult {
    /// Creates a successful import result.
    #[must_use] 
    pub const fn success(persona: Persona, tokens_imported: usize, warnings: Vec<String>) -> Self {
        Self {
            success: true,
            persona: Some(persona),
            tokens_imported,
            warnings,
            error: None,
        }
    }

    /// Creates a failed import result with an error message.
    #[must_use] 
    pub const fn failure(error: String) -> Self {
        Self {
            success: false,
            persona: None,
            tokens_imported: 0,
            warnings: vec![],
            error: Some(error),
        }
    }
}

/// Options controlling import behavior.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportOptions {
    /// How to handle name conflicts with existing personas
    pub on_conflict: ImportConflictStrategy,
    /// Whether to import granularity levels (currently ignored; levels are hardcoded)
    pub import_granularities: bool,
}

/// Strategy for resolving name conflicts during import.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImportConflictStrategy {
    /// Skip import if a persona with the same name exists
    #[default]
    Skip,
    /// Rename the imported persona by appending "(Imported N)"
    Rename,
    /// Delete the existing persona and import the new one
    Replace,
}
