//! Tauri IPC Command Handlers
//!
//! This module contains all Tauri commands that expose backend functionality to the frontend
//! via inter-process communication (IPC). Each command is a thin wrapper that:
//!
//! 1. Extracts the database connection from application state
//! 2. Delegates to repository or service functions
//! 3. Returns serializable results or errors
//!
//! # Module Organization
//!
//! Commands are organized by domain to maintain separation of concerns:
//!
//! - [`persona`]: CRUD operations for persona entities and generation parameters
//! - [`token`]: Token management including batch creation and reordering
//! - [`prompt`]: Prompt composition from persona tokens
//! - [`tokenizer`]: Model-aware token counting for prompt length validation
//! - [`ai`]: AI-powered token generation using LLM providers
//! - [`export`]: Persona import/export for backup and sharing
//! - [`settings`]: API key management via secure OS credential storage
//!
//! # Error Handling
//!
//! All commands return `Result<T, AppError>` where `AppError` implements `Serialize`
//! for Tauri IPC compatibility. Errors are propagated to the frontend for user feedback.

pub mod ai;
pub mod config;
pub mod export;
pub mod persona;
pub mod prompt;
pub mod settings;
pub mod token;
pub mod tokenizer;
