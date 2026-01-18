/**
 * Configuration service - Tauri IPC wrapper for application configuration
 *
 * This module provides access to backend configuration constants, ensuring
 * the frontend uses the same values as the Rust backend without duplication.
 *
 * @module services/config
 */

import { tauriInvoke } from './tauri';

/**
 * Retrieves the default image generation model identifier from the backend.
 *
 * This function fetches the constant from the Rust backend rather than
 * hardcoding it in TypeScript, ensuring a single source of truth.
 *
 * @returns Promise resolving to the HuggingFace model identifier
 *          (e.g., "stabilityai/stable-diffusion-xl-base-1.0")
 *
 * @example
 * ```typescript
 * const defaultModel = await getDefaultImageModelId();
 * // Use defaultModel as initial form value or fallback
 * ```
 *
 * @see src-tauri/src/domain/constants.rs - Rust source of truth
 */
export async function getDefaultImageModelId(): Promise<string> {
	return tauriInvoke<string>('get_default_image_model_id');
}

/**
 * Retrieves the list of valid AI provider identifiers from the backend.
 *
 * This function fetches the provider IDs from the Rust backend rather than
 * hardcoding them in TypeScript, ensuring a single source of truth.
 *
 * @returns Promise resolving to array of provider ID strings
 *          (e.g., ["openai", "anthropic", "google", "xai", "ollama"])
 *
 * @example
 * ```typescript
 * const providerIds = await getAiProviderIds();
 * // Use providerIds for validation or iteration
 * ```
 *
 * @see src-tauri/src/domain/ai.rs - Rust source of truth
 */
export async function getAiProviderIds(): Promise<string[]> {
	return tauriInvoke<string[]>('list_ai_provider_ids');
}
