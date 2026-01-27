/**
 * Token types - TypeScript equivalents of Rust domain types
 */

import type { ISODateString, UUID } from './common';

/** Token polarity - whether the token is positive (include) or negative (exclude) */
export type TokenPolarity = 'positive' | 'negative';

/** A granularity level for organizing tokens (e.g., Hair, Face, Upper Body) */
export interface GranularityLevel {
	id: string;
	name: string;
	/** DaisyUI color name for styling (e.g., "primary", "secondary") */
	color: string;
	display_order: number;
	is_default: boolean;
	created_at: ISODateString;
}

/** A token represents a single descriptive element for image generation */
export interface Token {
	id: UUID;
	persona_id: UUID;
	granularity_id: string;
	polarity: TokenPolarity;
	content: string;
	weight: number;
	display_order: number;
	created_at: ISODateString;
	updated_at: ISODateString;
}

/** Request to create a new token */
export interface CreateTokenRequest {
	persona_id: UUID;
	granularity_id: string;
	polarity: TokenPolarity;
	content: string;
	weight?: number;
}

/** Request to create multiple tokens at once */
export interface BatchCreateTokenRequest {
	persona_id: UUID;
	granularity_id: string;
	polarity: TokenPolarity;
	/** Comma-separated token contents */
	contents: string;
	weight?: number;
}

/** Request to update an existing token */
export interface UpdateTokenRequest {
	content?: string;
	weight?: number;
	granularity_id?: string;
	polarity?: TokenPolarity;
}

/** Single token ordering update within a reorder request */
export interface TokenOrderUpdate {
	/** Token UUID */
	token_id: string;
	/** New global display order position */
	display_order: number;
}

/** Request to reorder tokens within a persona */
export interface ReorderTokensRequest {
	/** Parent persona UUID */
	persona_id: string;
	/** Token ID to display_order mappings */
	token_orders: TokenOrderUpdate[];
}

/**
 * Represents a pending token operation during edit mode.
 *
 * Operations are staged locally and only persisted when the user saves.
 * Each operation type stores the data needed to execute or rollback:
 * - 'create': New token with a temporary ID (replaced with real ID on save)
 * - 'update': Tracks original data for rollback and the specific field changes
 * - 'delete': Stores original data to restore if edit is cancelled
 * - 'reorder': Batch update of display_order values from drag-and-drop
 *
 * @example
 * // Create operation
 * { type: 'create', tempId: 'temp-123', data: { id: 'temp-123', content: 'blue eyes', ... } }
 *
 * // Update operation
 * { type: 'update', id: 'real-id', originalData: {...}, updates: { weight: 1.5 } }
 *
 * // Delete operation
 * { type: 'delete', id: 'real-id', originalData: {...} }
 *
 * // Reorder operation
 * { type: 'reorder', orders: [{ token_id: 'id1', display_order: 0 }, ...] }
 */
export type DraftTokenOperation =
	| { type: 'create'; tempId: string; data: Omit<Token, 'created_at' | 'updated_at'> }
	| { type: 'update'; id: string; originalData: Token; updates: UpdateTokenRequest }
	| { type: 'delete'; id: string; originalData: Token }
	| { type: 'reorder'; orders: TokenOrderUpdate[] };

/**
 * Draft state for token editing session.
 *
 * Workflow:
 * 1. User clicks Edit → startDraft() copies current tokens to originalTokens
 * 2. User makes changes → operations added to pendingOperations, draftTokens updated
 * 3. User clicks Save → commitDraft() applies pendingOperations to backend
 * 4. User clicks Cancel → discardDraft() restores originalTokens
 *
 * The draftTokens array always reflects the current visual state by applying
 * pendingOperations to originalTokens (creates added, updates applied, deletes removed).
 */
export interface TokenDraftState {
	/** Original tokens at the start of editing (for rollback reference) */
	originalTokens: Token[];
	/** Pending operations to apply on save */
	pendingOperations: DraftTokenOperation[];
	/** Current view state - computed from original + pending */
	draftTokens: Token[];
}
