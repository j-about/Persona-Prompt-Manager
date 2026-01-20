//! Persona Prompt Manager - Rust Backend Library
//!
//! This library provides the core backend functionality for the Persona Prompt Manager
//! desktop application, a tool for managing AI image generation personas and composing
//! sophisticated prompts with granular control.
//!
//! # Architecture
//!
//! The application follows a clean architecture pattern with three primary layers:
//!
//! - **Commands Layer** ([`commands`]): Tauri IPC handlers that expose backend functionality
//!   to the frontend. These are thin wrappers that delegate to domain and infrastructure layers.
//!
//! - **Domain Layer** ([`domain`]): Core business logic and entity definitions. Contains
//!   the canonical representations of personas, tokens, prompts, and AI configuration.
//!
//! - **Infrastructure Layer** ([`infrastructure`]): External integrations including `SQLite`
//!   database operations, AI provider adapters, `HuggingFace` tokenizer integration, and
//!   secure credential storage via the OS keyring.
//!
//! # Key Features
//!
//! - **Persona Management**: Create, edit, and organize character profiles with metadata
//! - **Token Organization**: Hierarchical token management with granularity levels and polarity
//! - **Prompt Composition**: Assemble prompts from tokens with weight modifiers
//! - **Multi-Model Tokenization**: Accurate token counting for SDXL, FLUX, and other models
//! - **AI Token Generation**: Generate tokens using `OpenAI`, Anthropic, Google, xAI, or Ollama
//! - **Secure Credentials**: Platform-native secure storage for API keys

pub mod commands;
pub mod domain;
pub mod error;
pub mod infrastructure;

use std::sync::Mutex;
use tauri::Manager;

use infrastructure::Database;

/// Thread-safe application state shared across all Tauri command invocations.
///
/// This struct is managed by Tauri and injected into commands via the `State` extractor.
/// The database connection is wrapped in a `Mutex` to ensure safe concurrent access
/// from multiple frontend requests.
pub struct AppState {
    /// `SQLite` database connection wrapped in a mutex for thread-safe access.
    pub db: Mutex<Database>,
}

/// Initializes and runs the Tauri application.
///
/// This function performs the following initialization sequence:
/// 1. Registers Tauri plugins for process control and OS detection
/// 2. Creates the app data directory and initializes `SQLite` with WAL mode
/// 3. Stores the database connection in Tauri's managed state
/// 4. Registers all IPC command handlers
///
/// # Panics
///
/// Panics if the app data directory cannot be created or the database fails to initialize.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

            let db_path = app_data_dir.join("ppm.db");
            let database = Database::new(&db_path).expect("Failed to initialize database");

            app.manage(AppState {
                db: Mutex::new(database),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Persona commands
            commands::persona::create_persona,
            commands::persona::get_persona_by_id,
            commands::persona::list_personas,
            commands::persona::search_personas,
            commands::persona::update_persona,
            commands::persona::delete_persona,
            commands::persona::get_persona_generation_params,
            commands::persona::update_generation_params,
            commands::persona::duplicate_persona,
            // Token commands
            commands::token::create_token,
            commands::token::create_tokens_batch,
            commands::token::get_tokens_by_persona,
            commands::token::update_token,
            commands::token::delete_token,
            commands::token::reorder_tokens,
            commands::token::get_all_granularity_levels,
            // Prompt commands
            commands::prompt::compose_prompt,
            // Tokenizer commands
            commands::tokenizer::count_tokens_for_model,
            commands::tokenizer::get_known_image_models,
            // AI commands
            commands::ai::generate_ai_token_suggestions,
            commands::ai::generate_persona_with_ai,
            commands::ai::get_ai_provider_config,
            commands::ai::get_ai_provider_metadata,
            // Export/Import commands
            commands::export::export_all_personas,
            commands::export::import_personas,
            commands::export::parse_import_json,
            // Settings commands (including keyring)
            commands::settings::store_api_key,
            commands::settings::get_api_key_for_provider,
            commands::settings::delete_api_key,
            commands::settings::get_api_key_status,
            commands::settings::check_credential_store,
            // Configuration commands
            commands::config::get_default_image_model_id,
            commands::config::list_ai_provider_ids,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
