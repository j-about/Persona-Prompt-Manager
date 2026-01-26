/**
 * Config store - Svelte 5 runes-based state management for app configuration
 *
 * This store fetches configuration data from the Rust backend (single source of truth)
 * and caches it for use throughout the frontend.
 */

import { getAiProviderMetadata } from '$lib/services/ai';
import { getKnownModels } from '$lib/services/tokenizer';
import type { AiProviderMetadata, TokenizerInfo } from '$lib/types';

/** Create a reactive config store */
function createConfigStore() {
	// AI Provider state
	let aiProviders = $state<AiProviderMetadata[]>([]);
	let aiProvidersLoaded = false;

	// Image Model state
	let imageModels = $state<TokenizerInfo[]>([]);
	let imageModelsLoaded = false;

	// Derived lookup helpers
	const aiProviderMap = $derived(new Map(aiProviders.map((p) => [p.id, p])));

	/** Load AI provider metadata from Rust backend */
	async function loadAiProviders(): Promise<void> {
		if (aiProvidersLoaded) return;

		try {
			aiProviders = await getAiProviderMetadata();
			aiProvidersLoaded = true;
		} catch (err) {
			console.error('Failed to load AI providers:', err);
		}
	}

	/** Load image model configurations from Rust backend */
	async function loadImageModels(): Promise<void> {
		if (imageModelsLoaded) return;

		try {
			imageModels = await getKnownModels();
			imageModelsLoaded = true;
		} catch (err) {
			console.error('Failed to load image models:', err);
		}
	}

	/** Initialize all configuration - call this at app startup */
	async function initialize(): Promise<void> {
		await Promise.all([loadAiProviders(), loadImageModels()]);
	}

	/** Get AI provider by ID */
	function getProviderById(id: string): AiProviderMetadata | undefined {
		return aiProviderMap.get(id);
	}

	return {
		// State getters
		get aiProviders() {
			return aiProviders;
		},
		get imageModels() {
			return imageModels;
		},

		// Actions
		initialize,

		// Lookup helpers
		getProviderById
	};
}

// Export a singleton instance
export const configStore = createConfigStore();
