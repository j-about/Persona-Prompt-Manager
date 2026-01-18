/**
 * Settings service - Tauri IPC wrapper for API key management
 *
 * Provides secure API key storage and retrieval via OS keyring.
 */

import { tauriInvoke } from './tauri';
import type { AiProvider } from '$lib/types';

/** Status of an API key for a provider */
export interface ApiKeyStatus {
	provider: AiProvider;
	has_key: boolean;
}

/**
 * Store an API key securely in the OS keyring
 *
 * @param provider - The AI provider ID
 * @param apiKey - The API key to store
 */
export async function storeApiKey(provider: AiProvider, apiKey: string): Promise<void> {
	return tauriInvoke('store_api_key', { provider, apiKey });
}

/**
 * Get an API key from the OS keyring
 *
 * @param provider - The AI provider ID
 * @returns The API key or null if not set
 */
export async function getApiKey(provider: AiProvider): Promise<string | null> {
	return tauriInvoke<string | null>('get_api_key_for_provider', { provider });
}

/**
 * Delete an API key from the OS keyring
 *
 * @param provider - The AI provider ID
 */
export async function deleteApiKey(provider: AiProvider): Promise<void> {
	return tauriInvoke('delete_api_key', { provider });
}

/**
 * Get the status of API keys for all providers
 *
 * @returns Array of provider statuses indicating which have keys configured
 */
export async function getApiKeyStatus(): Promise<ApiKeyStatus[]> {
	return tauriInvoke<ApiKeyStatus[]>('get_api_key_status');
}

/**
 * Check if the system credential store is available
 * On Linux, returns false if no Secret Service daemon is running
 *
 * @returns true if the credential store is available
 */
export async function checkCredentialStore(): Promise<boolean> {
	return tauriInvoke<boolean>('check_credential_store');
}
