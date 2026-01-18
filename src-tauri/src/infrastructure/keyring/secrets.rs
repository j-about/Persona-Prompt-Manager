//! Secrets management using OS keyring
//!
//! Provides secure storage and retrieval of API keys using the
//! operating system's native credential store.

use keyring::Entry;

use crate::domain::ai::AiProvider;
use crate::error::AppError;

/// Service name for keyring entries
const SERVICE_NAME: &str = "persona-prompt-manager";

/// Build the keyring entry name for an AI provider
fn build_keyring_entry_name(provider: &AiProvider) -> String {
    format!("api-key-{}", provider_to_string_id(provider))
}

/// Convert provider enum to string ID
const fn provider_to_string_id(provider: &AiProvider) -> &'static str {
    match provider {
        AiProvider::OpenAI => "openai",
        AiProvider::Anthropic => "anthropic",
        AiProvider::Google => "google",
        AiProvider::XAi => "xai",
        AiProvider::Ollama => "ollama",
    }
}

/// Store an API key securely in the OS keyring
pub fn store_api_key(provider: &AiProvider, api_key: &str) -> Result<(), AppError> {
    let entry_name = build_keyring_entry_name(provider);
    let entry = Entry::new(SERVICE_NAME, &entry_name)
        .map_err(|e| AppError::Internal(format!("Failed to create keyring entry: {e}")))?;

    entry
        .set_password(api_key)
        .map_err(|e| AppError::Internal(format!("Failed to store API key in keyring: {e}")))?;

    Ok(())
}

/// Retrieve an API key from the OS keyring
pub fn get_api_key(provider: &AiProvider) -> Result<Option<String>, AppError> {
    let entry_name = build_keyring_entry_name(provider);
    let entry = Entry::new(SERVICE_NAME, &entry_name)
        .map_err(|e| AppError::Internal(format!("Failed to create keyring entry: {e}")))?;

    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Internal(format!(
            "Failed to retrieve API key from keyring: {e}"
        ))),
    }
}

/// Delete an API key from the OS keyring
pub fn delete_api_key(provider: &AiProvider) -> Result<(), AppError> {
    let entry_name = build_keyring_entry_name(provider);
    let entry = Entry::new(SERVICE_NAME, &entry_name)
        .map_err(|e| AppError::Internal(format!("Failed to create keyring entry: {e}")))?;

    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // Already deleted, that's fine
        Err(e) => Err(AppError::Internal(format!(
            "Failed to delete API key from keyring: {e}"
        ))),
    }
}

/// Check if an API key exists in the keyring for a provider
pub fn has_api_key(provider: &AiProvider) -> Result<bool, AppError> {
    match get_api_key(provider) {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Ok(false),
        Err(e) => Err(e),
    }
}

/// Get all providers with stored API keys (provider → has key)
pub fn get_providers_with_stored_keys() -> Result<Vec<(AiProvider, bool)>, AppError> {
    let mut results = Vec::new();

    for provider in AiProvider::all() {
        let has_key = has_api_key(provider)?;
        results.push((*provider, has_key));
    }

    Ok(results)
}

/// Check if the credential store backend is available
/// On Linux, this checks if the Secret Service (gnome-keyring, kwallet, etc.) is running
/// On macOS/Windows, this always returns true as they have built-in credential stores
pub fn check_credential_store_available() -> Result<bool, AppError> {
    #[cfg(target_os = "linux")]
    {
        // Try to read from keyring to verify Secret Service is actually available
        // Entry::new() succeeds even without a secret service - we need to try an operation
        let test_entry = match Entry::new("persona-prompt-manager-test", "availability-check") {
            Ok(entry) => entry,
            Err(keyring::Error::NoStorageAccess(_)) => return Ok(false),
            Err(keyring::Error::PlatformFailure(_)) => return Ok(false),
            Err(e) => {
                return Err(AppError::Internal(format!(
                    "Failed to check credential store: {e}"
                )))
            }
        };

        // Actually try to read - this will fail if secret service is not running
        match test_entry.get_password() {
            Ok(_) => Ok(true),                        // Entry exists, service is working
            Err(keyring::Error::NoEntry) => Ok(true), // No entry but service is working
            Err(keyring::Error::NoStorageAccess(_)) => Ok(false),
            Err(keyring::Error::PlatformFailure(_)) => Ok(false),
            Err(e) => Err(AppError::Internal(format!(
                "Failed to check credential store: {e}"
            ))),
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        // macOS and Windows have built-in credential stores that are always available
        Ok(true)
    }
}
