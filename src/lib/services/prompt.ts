/**
 * Prompt service - Tauri IPC wrapper for prompt composition operations
 */

import { tauriInvoke } from './tauri';
import type { ComposedPrompt, CompositionOptions } from '$lib/types';

/** Compose a prompt from a persona's tokens */
export async function composePrompt(
	personaId: string,
	options?: CompositionOptions
): Promise<ComposedPrompt> {
	return tauriInvoke<ComposedPrompt>('compose_prompt', { personaId, options });
}

/** Copy text to clipboard */
export async function copyToClipboard(text: string): Promise<void> {
	try {
		await navigator.clipboard.writeText(text);
	} catch (error) {
		console.error('Failed to copy to clipboard:', error);
		throw error;
	}
}
