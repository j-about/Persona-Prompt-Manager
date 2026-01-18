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
	let aiProvidersLoaded = $state(false);
	let aiProvidersLoading = $state(false);
	let aiProvidersError = $state<string | null>(null);

	// Image Model state
	let imageModels = $state<TokenizerInfo[]>([]);
	let imageModelsLoaded = $state(false);
	let imageModelsLoading = $state(false);
	let imageModelsError = $state<string | null>(null);

	// Derived lookup helpers
	const aiProviderMap = $derived(new Map(aiProviders.map((p) => [p.id, p])));

	const imageModelMap = $derived(new Map(imageModels.map((m) => [m.model_id, m])));

	/** Load AI provider metadata from Rust backend */
	async function loadAiProviders(): Promise<void> {
		if (aiProvidersLoaded || aiProvidersLoading) return;

		aiProvidersLoading = true;
		aiProvidersError = null;

		try {
			aiProviders = await getAiProviderMetadata();
			aiProvidersLoaded = true;
		} catch (err) {
			aiProvidersError = err instanceof Error ? err.message : 'Failed to load AI providers';
			console.error('Failed to load AI providers:', err);
		} finally {
			aiProvidersLoading = false;
		}
	}

	/** Load image model configurations from Rust backend */
	async function loadImageModels(): Promise<void> {
		if (imageModelsLoaded || imageModelsLoading) return;

		imageModelsLoading = true;
		imageModelsError = null;

		try {
			imageModels = await getKnownModels();
			imageModelsLoaded = true;
		} catch (err) {
			imageModelsError = err instanceof Error ? err.message : 'Failed to load image models';
			console.error('Failed to load image models:', err);
		} finally {
			imageModelsLoading = false;
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

	/** Get image model by model ID */
	function getModelById(modelId: string): TokenizerInfo | undefined {
		return imageModelMap.get(modelId);
	}

	/** Check if a provider ID is valid (exists in backend-defined list) */
	function isValidProviderId(id: string): boolean {
		return aiProviderMap.has(id);
	}

	return {
		// AI Provider state getters
		get aiProviders() {
			return aiProviders;
		},
		get aiProvidersLoaded() {
			return aiProvidersLoaded;
		},
		get aiProvidersLoading() {
			return aiProvidersLoading;
		},
		get aiProvidersError() {
			return aiProvidersError;
		},

		// Image Model state getters
		get imageModels() {
			return imageModels;
		},
		get imageModelsLoaded() {
			return imageModelsLoaded;
		},
		get imageModelsLoading() {
			return imageModelsLoading;
		},
		get imageModelsError() {
			return imageModelsError;
		},

		// Actions
		loadAiProviders,
		loadImageModels,
		initialize,

		// Lookup helpers
		getProviderById,
		getModelById,
		isValidProviderId
	};
}

// Export a singleton instance
export const configStore = createConfigStore();
