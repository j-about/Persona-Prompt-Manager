//! Settings and Credential Management Commands
//!
//! This module provides Tauri IPC commands for secure API key management using
//! the operating system's native credential storage:
//!
//! - **macOS**: Keychain
//! - **Windows**: Windows Credential Manager
//! - **Linux**: Secret Service (GNOME Keyring, `KWallet`, etc.)
//!
//! # Security Model
//!
//! API keys are never stored in the application database or configuration files.
//! The OS keyring provides:
//! - Encryption at rest
//! - Access control tied to user session
//! - Protection from unauthorized process access
//!
//! # Linux Requirements
//!
//! Linux requires a Secret Service daemon (gnome-keyring or kwallet) to be running.
//! The `check_credential_store` command allows the application to detect this
//! and show appropriate guidance to users.

use crate::domain::ai::AiProvider;
use crate::error::AppError;
use crate::infrastructure::keyring;

/// Stores an API key securely in the OS credential store.
///
/// Overwrites any existing key for the same provider. The key is stored
/// with an entry name based on the provider ID (e.g., "api-key-openai").
///
/// # Arguments
///
/// * `provider` - The AI provider this key authenticates to
/// * `api_key` - The API key value to store
///
/// # Errors
///
/// Returns `AppError::Internal` if the credential store is unavailable or
/// the storage operation fails.
#[tauri::command]
pub fn store_api_key(provider: AiProvider, api_key: String) -> Result<(), AppError> {
    keyring::store_api_key(&provider, &api_key)
}

/// Retrieves an API key from the OS credential store for a specific provider.
///
/// # Arguments
///
/// * `provider` - The AI provider whose key to retrieve
///
/// # Returns
///
/// The API key if one is stored, or `None` if no key exists for this provider.
///
/// # Errors
///
/// Returns `AppError::Internal` if the credential store is unavailable or
/// the retrieval operation fails.
#[tauri::command]
pub fn get_api_key_for_provider(provider: AiProvider) -> Result<Option<String>, AppError> {
    keyring::get_api_key(&provider)
}

/// Deletes an API key from the OS credential store.
///
/// Silently succeeds if no key exists for the provider.
///
/// # Arguments
///
/// * `provider` - The AI provider whose key to delete
///
/// # Errors
///
/// Returns `AppError::Internal` if the credential store is unavailable or
/// the deletion operation fails.
#[tauri::command]
pub fn delete_api_key(provider: AiProvider) -> Result<(), AppError> {
    keyring::delete_api_key(&provider)
}

/// Status information for an API key.
///
/// Used by the frontend to show which providers have keys configured
/// without exposing the actual key values.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiKeyStatus {
    /// The AI provider this status applies to
    pub provider: AiProvider,
    /// Whether an API key is stored for this provider
    pub has_key: bool,
}

/// Returns the API key status for all supported providers.
///
/// This allows the frontend to display configuration status without
/// retrieving actual key values, following the principle of least privilege.
///
/// # Returns
///
/// Vector of `ApiKeyStatus` for all providers (`OpenAI`, Anthropic, Google, xAI, Ollama).
#[tauri::command]
pub fn get_api_key_status() -> Result<Vec<ApiKeyStatus>, AppError> {
    let stored = keyring::get_providers_with_stored_keys()?;

    Ok(stored
        .into_iter()
        .map(|(provider, has_key)| ApiKeyStatus { provider, has_key })
        .collect())
}

/// Checks if the system credential store is available and functional.
///
/// On macOS and Windows, this always returns `true` as these platforms have
/// built-in credential storage. On Linux, it verifies that a Secret Service
/// daemon (gnome-keyring, kwallet, etc.) is running and accessible.
///
/// # Returns
///
/// `true` if credential storage is available, `false` otherwise.
///
/// # Usage
///
/// The application calls this at startup on Linux to detect missing keyring
/// services and display setup instructions to the user.
#[tauri::command]
pub fn check_credential_store() -> Result<bool, AppError> {
    keyring::check_credential_store_available()
}
