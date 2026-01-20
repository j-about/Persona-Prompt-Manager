//! Domain Layer - Business Logic and Core Entities
//!
//! This module contains the canonical domain models and business logic that define
//! the application's core functionality. All other layers (commands, infrastructure)
//! depend on these definitions.
//!
//! # Domain Model Overview
//!
//! The application models AI image generation workflows with these core concepts:
//!
//! - **Personas**: Character profiles that organize related generation data
//! - **Tokens**: Atomic descriptive elements with weight and polarity
//! - **Prompts**: Composed output ready for image generation tools
//! - **AI Configuration**: Provider settings for LLM-based token generation
//!
//! # Module Organization
//!
//! - [`persona`]: Persona entities and generation parameters
//! - [`token`]: Token entities, granularity levels, and polarity
//! - [`prompt`]: Prompt composition logic and output formatting
//! - [`ai`]: AI provider configuration and token generation types
//! - [`export`]: Import/export data structures for backup and sharing
//!
//! # Design Principles
//!
//! - **Serialization**: All types implement `Serialize`/`Deserialize` for Tauri IPC
//! - **Immutable by Default**: Updates are explicit via `update()` methods
//! - **Validation at Boundaries**: Domain types trust their invariants internally

pub mod ai;
pub mod constants;
pub mod export;
pub mod persona;
pub mod prompt;
pub mod token;

// Re-export commonly used types for ergonomic imports
pub use ai::{
    AiProvider, AiProviderConfig, AiProviderStatus, GeneratedToken, TokenGenerationRequest,
    TokenGenerationResponse,
};
pub use export::{BulkExport, ImportConflictStrategy, ImportOptions, ImportResult, PersonaExport};
pub use persona::{CreatePersonaRequest, GenerationParams, Persona, UpdatePersonaRequest};
pub use prompt::{ComposePromptRequest, ComposedPrompt, CompositionOptions, PromptComposer};
pub use token::{
    BatchCreateTokenRequest, CreateTokenRequest, Granularity, GranularityLevel, Token,
    TokenPolarity, UpdateTokenRequest,
};

// Re-export domain constants for convenient access
pub use constants::DEFAULT_IMAGE_MODEL_ID;
