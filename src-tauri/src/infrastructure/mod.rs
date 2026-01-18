//! Infrastructure Layer - External Integrations
//!
//! This module contains all external integrations and I/O operations,
//! encapsulating the details of how the application interacts with:
//!
//! - **Database**: `SQLite` persistence with WAL mode for performance
//! - **AI Providers**: LLM integrations for token generation (`OpenAI`, Anthropic, etc.)
//! - **Tokenizer**: `HuggingFace` tokenizers for accurate prompt length calculation
//! - **Keyring**: Platform-native secure credential storage
//!
//! # Architecture Role
//!
//! The infrastructure layer sits at the bottom of the dependency hierarchy.
//! Domain and command layers depend on infrastructure, but infrastructure
//! has no dependencies on upper layers (except for domain types as parameters).
//!
//! # Module Organization
//!
//! - [`database`]: `SQLite` connection management, migrations, and repositories
//! - [`ai`]: Multi-provider AI adapter using the `genai` crate
//! - [`tokenizer`]: Model-aware token counting for CLIP and T5 tokenizers
//! - [`keyring`]: Secure API key storage using OS credential managers

pub mod ai;
pub mod database;
pub mod keyring;
pub mod tokenizer;

// Re-export commonly used types for ergonomic imports
pub use database::Database;
pub use keyring::{delete_api_key, get_api_key, has_api_key, store_api_key};
pub use tokenizer::{
    count_tokens, count_tokens_batch, get_config_for_model, get_known_models, get_tokenizer_info,
    TokenCount, TokenizerConfig, TokenizerInfo,
};
