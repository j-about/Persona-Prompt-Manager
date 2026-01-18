/**
 * Token store - Svelte 5 runes-based state management for tokens
 *
 * Manages tokens for the currently selected persona, organized by granularity level.
 */

import { SvelteMap } from 'svelte/reactivity';
import * as tokenService from '$lib/services/token';
import type {
	Token,
	GranularityLevel,
	CreateTokenRequest,
	BatchCreateTokenRequest,
	UpdateTokenRequest,
	TokenPolarity,
	TokenDraftState
} from '$lib/types';

/** Create a reactive token store */
function createTokenStore() {
	// Reactive state
	let tokens = $state<Token[]>([]);
	let granularityLevels = $state<GranularityLevel[]>([]);
	let currentPersonaId = $state<string | null>(null);
	let isLoading = $state(false);
	let error = $state<string | null>(null);

	// Draft mode state for edit sessions
	let draftState = $state<TokenDraftState | null>(null);

	// Helper to generate temp IDs for draft tokens
	function generateTempId(): string {
		return `temp_${crypto.randomUUID()}`;
	}

	// Derived: tokens grouped by granularity and polarity
	const tokensByGranularity = $derived(() => {
		const grouped = new SvelteMap<string, { positive: Token[]; negative: Token[] }>();

		// Initialize with all granularity levels
		for (const level of granularityLevels) {
			grouped.set(level.id, { positive: [], negative: [] });
		}

		// Sort tokens by display order and group
		const sortedTokens = [...tokens].sort((a, b) => a.display_order - b.display_order);

		for (const token of sortedTokens) {
			const group = grouped.get(token.granularity_id);
			if (group) {
				if (token.polarity === 'positive') {
					group.positive.push(token);
				} else {
					group.negative.push(token);
				}
			}
		}

		return grouped;
	});

	// Derived: sorted granularity levels
	const sortedGranularityLevels = $derived(
		[...granularityLevels].sort((a, b) => a.display_order - b.display_order)
	);

	// Derived: token counts
	const positiveCount = $derived(tokens.filter((t) => t.polarity === 'positive').length);
	const negativeCount = $derived(tokens.filter((t) => t.polarity === 'negative').length);
	const totalCount = $derived(tokens.length);

	// Actions
	async function loadGranularityLevels(): Promise<void> {
		try {
			granularityLevels = await tokenService.getGranularityLevels();
		} catch (err) {
			console.error('Failed to load granularity levels:', err);
		}
	}

	async function loadTokensForPersona(personaId: string): Promise<void> {
		// Clear any existing draft state when switching personas
		if (draftState) {
			draftState = null;
		}

		isLoading = true;
		error = null;
		currentPersonaId = personaId;

		try {
			// Load both granularity levels and tokens
			const [levels, personaTokens] = await Promise.all([
				tokenService.getGranularityLevels(),
				tokenService.getTokensByPersona(personaId)
			]);

			granularityLevels = levels;
			tokens = personaTokens;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load tokens';
			console.error('Failed to load tokens:', err);
		} finally {
			isLoading = false;
		}
	}

	async function createToken(request: CreateTokenRequest): Promise<Token | null> {
		isLoading = true;
		error = null;

		try {
			const newToken = await tokenService.createToken(request);
			tokens = [...tokens, newToken];
			return newToken;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to create token';
			console.error('Failed to create token:', err);
			return null;
		} finally {
			isLoading = false;
		}
	}

	async function createTokensBatch(request: BatchCreateTokenRequest): Promise<Token[]> {
		isLoading = true;
		error = null;

		try {
			const newTokens = await tokenService.createTokensBatch(request);
			tokens = [...tokens, ...newTokens];
			return newTokens;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to create tokens';
			console.error('Failed to create tokens:', err);
			return [];
		} finally {
			isLoading = false;
		}
	}

	async function updateToken(id: string, request: UpdateTokenRequest): Promise<Token | null> {
		isLoading = true;
		error = null;

		try {
			const updatedToken = await tokenService.updateToken(id, request);
			tokens = tokens.map((t) => (t.id === id ? updatedToken : t));
			return updatedToken;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to update token';
			console.error('Failed to update token:', err);
			return null;
		} finally {
			isLoading = false;
		}
	}

	async function deleteToken(id: string): Promise<boolean> {
		isLoading = true;
		error = null;

		try {
			await tokenService.deleteToken(id);
			tokens = tokens.filter((t) => t.id !== id);
			return true;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to delete token';
			console.error('Failed to delete token:', err);
			return false;
		} finally {
			isLoading = false;
		}
	}

	async function reorderTokens(tokenIds: string[]): Promise<boolean> {
		// Optimistically update the local state
		const originalTokens = [...tokens];
		const reorderedTokens = tokenIds
			.map((id, index) => {
				const token = tokens.find((t) => t.id === id);
				if (token) {
					return { ...token, display_order: index };
				}
				return null;
			})
			.filter((t): t is Token => t !== null);

		// Update tokens that were reordered
		tokens = tokens.map((t) => {
			const reordered = reorderedTokens.find((rt) => rt.id === t.id);
			return reordered || t;
		});

		try {
			await tokenService.reorderTokens(tokenIds);
			return true;
		} catch (err) {
			// Revert on error
			tokens = originalTokens;
			error = err instanceof Error ? err.message : 'Failed to reorder tokens';
			console.error('Failed to reorder tokens:', err);
			return false;
		}
	}

	function getTokensForGranularity(granularityId: string, polarity?: TokenPolarity): Token[] {
		return tokens
			.filter(
				(t) =>
					t.granularity_id === granularityId && (polarity === undefined || t.polarity === polarity)
			)
			.sort((a, b) => a.display_order - b.display_order);
	}

	function clearError(): void {
		error = null;
	}

	function reset(): void {
		tokens = [];
		currentPersonaId = null;
		error = null;
	}

	// ==================== Draft Mode Methods ====================

	/**
	 * Start a draft editing session - snapshots current state
	 */
	function startDraft(): void {
		if (draftState) return; // Already in draft mode

		draftState = {
			originalTokens: [...tokens],
			pendingOperations: [],
			draftTokens: [...tokens]
		};
	}

	/**
	 * Add multiple tokens to draft state from comma-separated input (does not persist)
	 */
	function draftCreateTokensBatch(request: BatchCreateTokenRequest): void {
		if (!draftState) return;

		const contents = request.contents
			.split(',')
			.map((c) => c.trim())
			.filter((c) => c.length > 0);

		for (const content of contents) {
			const tempId = generateTempId();

			const newToken: Omit<Token, 'created_at' | 'updated_at'> = {
				id: tempId,
				persona_id: request.persona_id,
				granularity_id: request.granularity_id,
				polarity: request.polarity,
				content,
				weight: request.weight ?? 1.0,
				display_order: draftState.draftTokens.length
			};

			draftState.pendingOperations = [
				...draftState.pendingOperations,
				{ type: 'create', tempId, data: newToken }
			];

			// Add to draft view with placeholder timestamps
			draftState.draftTokens = [
				...draftState.draftTokens,
				{
					...newToken,
					created_at: new Date().toISOString(),
					updated_at: new Date().toISOString()
				}
			];
		}
	}

	/**
	 * Update a token in draft state
	 */
	function draftUpdateToken(id: string, updates: UpdateTokenRequest): void {
		if (!draftState) return;

		// Find the token (could be original or temp)
		const existingToken = draftState.draftTokens.find((t) => t.id === id);
		if (!existingToken) return;

		// Check if this is a temp token (created during this session)
		const existingCreateOp = draftState.pendingOperations.find(
			(op) => op.type === 'create' && op.tempId === id
		);

		if (existingCreateOp && existingCreateOp.type === 'create') {
			// Update the create operation data
			const updatedData = { ...existingCreateOp.data };
			if (updates.content !== undefined) updatedData.content = updates.content;
			if (updates.weight !== undefined) updatedData.weight = updates.weight;
			if (updates.granularity_id !== undefined) updatedData.granularity_id = updates.granularity_id;
			if (updates.polarity !== undefined) updatedData.polarity = updates.polarity;

			draftState.pendingOperations = draftState.pendingOperations.map((op) =>
				op.type === 'create' && op.tempId === id ? { ...op, data: updatedData } : op
			);
		} else {
			// This is an original token - find or create update operation
			const existingUpdateOp = draftState.pendingOperations.find(
				(op) => op.type === 'update' && op.id === id
			);

			if (existingUpdateOp && existingUpdateOp.type === 'update') {
				// Merge updates
				draftState.pendingOperations = draftState.pendingOperations.map((op) =>
					op.type === 'update' && op.id === id
						? { ...op, updates: { ...op.updates, ...updates } }
						: op
				);
			} else {
				// Create new update operation
				const originalToken = draftState.originalTokens.find((t) => t.id === id);
				if (originalToken) {
					draftState.pendingOperations = [
						...draftState.pendingOperations,
						{ type: 'update', id, originalData: originalToken, updates }
					];
				}
			}
		}

		// Update draft view
		draftState.draftTokens = draftState.draftTokens.map((t) => {
			if (t.id !== id) return t;
			return {
				...t,
				...(updates.content !== undefined && { content: updates.content }),
				...(updates.weight !== undefined && { weight: updates.weight }),
				...(updates.granularity_id !== undefined && { granularity_id: updates.granularity_id }),
				...(updates.polarity !== undefined && { polarity: updates.polarity }),
				updated_at: new Date().toISOString()
			};
		});
	}

	/**
	 * Delete a token from draft state
	 */
	function draftDeleteToken(id: string): void {
		if (!draftState) return;

		// Check if this is a temp token (just remove the create operation)
		const createOpIndex = draftState.pendingOperations.findIndex(
			(op) => op.type === 'create' && op.tempId === id
		);

		if (createOpIndex !== -1) {
			// Simply remove the create operation - never persisted
			draftState.pendingOperations = draftState.pendingOperations.filter(
				(_, i) => i !== createOpIndex
			);
		} else {
			// This is an original token - add delete operation (or cancel update)
			const originalToken = draftState.originalTokens.find((t) => t.id === id);
			if (originalToken) {
				// Remove any existing update operation for this token
				draftState.pendingOperations = draftState.pendingOperations.filter(
					(op) => !(op.type === 'update' && op.id === id)
				);
				// Add delete operation
				draftState.pendingOperations = [
					...draftState.pendingOperations,
					{ type: 'delete', id, originalData: originalToken }
				];
			}
		}

		// Update draft view
		draftState.draftTokens = draftState.draftTokens.filter((t) => t.id !== id);
	}

	/**
	 * Commit all draft changes to backend
	 */
	async function commitDraft(): Promise<boolean> {
		if (!draftState) return true;

		isLoading = true;
		error = null;

		try {
			const operations = draftState.pendingOperations;

			// 1. Process deletes first
			const deleteOps = operations.filter((op) => op.type === 'delete');
			for (const op of deleteOps) {
				if (op.type === 'delete') {
					await tokenService.deleteToken(op.id);
				}
			}

			// 2. Process updates
			const updateOps = operations.filter((op) => op.type === 'update');
			for (const op of updateOps) {
				if (op.type === 'update') {
					await tokenService.updateToken(op.id, op.updates);
				}
			}

			// 3. Process creates
			const createOps = operations.filter((op) => op.type === 'create');
			for (const op of createOps) {
				if (op.type === 'create') {
					await tokenService.createToken({
						persona_id: op.data.persona_id,
						granularity_id: op.data.granularity_id,
						polarity: op.data.polarity,
						content: op.data.content,
						weight: op.data.weight
					});
				}
			}

			// Reload fresh data from backend
			if (currentPersonaId) {
				await loadTokensForPersona(currentPersonaId);
			}

			// Clear draft state
			draftState = null;

			return true;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to save token changes';
			console.error('Failed to commit draft:', err);
			return false;
		} finally {
			isLoading = false;
		}
	}

	/**
	 * Discard draft changes and restore original state
	 */
	function discardDraft(): void {
		if (!draftState) return;

		// Restore original tokens to the main state
		tokens = [...draftState.originalTokens];
		draftState = null;
	}

	return {
		// State getters
		get tokens() {
			return tokens;
		},
		get granularityLevels() {
			return sortedGranularityLevels;
		},
		get tokensByGranularity() {
			return tokensByGranularity;
		},
		get currentPersonaId() {
			return currentPersonaId;
		},
		get isLoading() {
			return isLoading;
		},
		get error() {
			return error;
		},
		get positiveCount() {
			return positiveCount;
		},
		get negativeCount() {
			return negativeCount;
		},
		get totalCount() {
			return totalCount;
		},

		// Draft mode state
		get isDraftMode() {
			return draftState !== null;
		},
		get hasPendingChanges() {
			return draftState !== null && draftState.pendingOperations.length > 0;
		},
		get draftTokens() {
			return draftState ? draftState.draftTokens : tokens;
		},

		// Actions
		loadGranularityLevels,
		loadTokensForPersona,
		createToken,
		createTokensBatch,
		updateToken,
		deleteToken,
		reorderTokens,
		getTokensForGranularity,
		clearError,
		reset,

		// Draft mode actions
		startDraft,
		draftCreateTokensBatch,
		draftUpdateToken,
		draftDeleteToken,
		commitDraft,
		discardDraft
	};
}

// Export a singleton instance
export const tokenStore = createTokenStore();
