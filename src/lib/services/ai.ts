/**
 * AI service - Tauri IPC wrapper for AI generation operations
 */

import { tauriInvoke } from './tauri';
import type {
	AiPersonaGenerationRequest,
	AiPersonaGenerationResponse,
	AiProvider,
	AiProviderConfig,
	AiProviderMetadata,
	TokenGenerationRequest,
	TokenGenerationResponse
} from '$lib/types';

// ============================================================================
// Persona Generation
// ============================================================================
//
// Creates complete persona profiles with tokens organized by body region.

/** Generate a complete persona using AI, including description, tags, and initial tokens */
export async function generatePersonaWithAi(
	config: AiProviderConfig,
	request: AiPersonaGenerationRequest
): Promise<AiPersonaGenerationResponse> {
	return tauriInvoke<AiPersonaGenerationResponse>('generate_persona_with_ai', { config, request });
}

// ============================================================================
// Token Generation
// ============================================================================
//
// Generates additional positive/negative tokens during prompt composition.

/** Generate token suggestions using an AI provider */
export async function generateTokens(
	config: AiProviderConfig,
	request: TokenGenerationRequest
): Promise<TokenGenerationResponse> {
	return tauriInvoke<TokenGenerationResponse>('generate_ai_token_suggestions', { config, request });
}

// ============================================================================
// Provider Configuration
// ============================================================================
//
// Utilities for configuring AI providers.

/** Get the default configuration for an AI provider */
export async function getAiProviderConfig(provider: AiProvider): Promise<AiProviderConfig> {
	return tauriInvoke<AiProviderConfig>('get_ai_provider_config', { provider });
}

/** Get complete metadata for all AI providers (from Rust single source of truth) */
export async function getAiProviderMetadata(): Promise<AiProviderMetadata[]> {
	return tauriInvoke<AiProviderMetadata[]>('get_ai_provider_metadata');
}
