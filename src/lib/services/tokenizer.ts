/**
 * Tokenizer service - Tauri IPC wrapper for token counting operations
 *
 * Provides model-aware token counting for various image generation models.
 */

import { tauriInvoke } from './tauri';
import type { TokenCount, TokenizerInfo } from '$lib/types';

/**
 * Count tokens in a text string for a specific model
 *
 * @param text - The text to count tokens for
 * @param modelId - Optional model ID. If not provided, uses the default model
 *                  from the backend (see `getDefaultImageModelId()` in config service).
 */
export async function countTokens(text: string, modelId?: string): Promise<TokenCount> {
	return tauriInvoke<TokenCount>('count_tokens_for_model', {
		text,
		model_id: modelId ?? null
	});
}

/**
 * Get list of all known model mappings
 *
 * Returns configurations for all models with known tokenizer mappings.
 */
export async function getKnownModels(): Promise<TokenizerInfo[]> {
	return tauriInvoke<TokenizerInfo[]>('get_known_image_models');
}
