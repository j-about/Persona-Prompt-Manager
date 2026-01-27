/**
 * Token service - Tauri IPC wrapper for token operations
 */

import { tauriInvoke } from './tauri';
import type {
	Token,
	CreateTokenRequest,
	UpdateTokenRequest,
	GranularityLevel,
	ReorderTokensRequest
} from '$lib/types';

/** Create a new token */
export async function createToken(request: CreateTokenRequest): Promise<Token> {
	return tauriInvoke<Token>('create_token', { request });
}

/** Get all tokens for a persona */
export async function getTokensByPersona(personaId: string): Promise<Token[]> {
	return tauriInvoke<Token[]>('get_tokens_by_persona', { personaId });
}

/** Update a token */
export async function updateToken(id: string, request: UpdateTokenRequest): Promise<Token> {
	return tauriInvoke<Token>('update_token', { id, request });
}

/** Delete a token */
export async function deleteToken(id: string): Promise<void> {
	return tauriInvoke<void>('delete_token', { id });
}

/** Get all granularity levels */
export async function getGranularityLevels(): Promise<GranularityLevel[]> {
	return tauriInvoke<GranularityLevel[]>('get_all_granularity_levels');
}

/** Reorder tokens within a persona (batch update display_order values) */
export async function reorderTokens(request: ReorderTokensRequest): Promise<void> {
	return tauriInvoke<void>('reorder_tokens', { request });
}
