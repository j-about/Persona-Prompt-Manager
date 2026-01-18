<!--
@component
TokenManager - Orchestrates token CRUD operations within a persona context.

Groups tokens by granularity level (Style, General, Hair, Face, etc.) and
polarity (positive/negative). Provides interfaces for adding, editing, and
deleting tokens through child components.
-->
<script lang="ts">
	import { SvelteMap } from 'svelte/reactivity';
	import type { Token, GranularityLevel, TokenPolarity, UpdateTokenRequest } from '$lib/types';
	import { Card, ConfirmDialog } from '$lib/components/ui';
	import TokenInput from './TokenInput.svelte';
	import TokenSection from './TokenSection.svelte';
	import TokenEditModal from './TokenEditModal.svelte';

	/**
	 * Component props for TokenManager.
	 * @property personaId - The ID of the persona owning these tokens
	 * @property tokens - Array of tokens to display and manage
	 * @property granularityLevels - Available granularity categories for grouping
	 * @property isLoading - Disables interactions during async operations
	 * @property isReadOnly - Hides editing controls when true
	 * @property onCreateToken - Callback for creating new tokens
	 * @property onUpdateToken - Callback for updating existing tokens
	 * @property onDeleteToken - Callback for deleting tokens
	 */
	interface Props {
		personaId: string;
		tokens: Token[];
		granularityLevels: GranularityLevel[];
		isLoading?: boolean;
		isReadOnly?: boolean;
		onCreateToken?: (data: {
			personaId: string;
			granularityId: string;
			polarity: TokenPolarity;
			contents: string;
			weight: number;
		}) => Promise<void>;
		onUpdateToken?: (id: string, data: UpdateTokenRequest) => Promise<void>;
		onDeleteToken?: (id: string) => Promise<void>;
	}

	let {
		personaId,
		tokens,
		granularityLevels,
		isLoading = false,
		isReadOnly = false,
		onCreateToken,
		onUpdateToken,
		onDeleteToken
	}: Props = $props();

	/** Token currently being edited in the modal, null if no edit in progress */
	let editingToken = $state<Token | null>(null);
	/** Controls visibility of the delete confirmation dialog */
	let showDeleteConfirm = $state(false);
	/** Token pending deletion confirmation */
	let tokenToDelete = $state<Token | null>(null);

	/**
	 * Groups tokens by granularity level and polarity.
	 * Creates a map where each granularity ID maps to positive/negative token arrays,
	 * sorted by display_order for consistent rendering.
	 */
	const tokensByGranularity = $derived(() => {
		const grouped = new SvelteMap<string, { positive: Token[]; negative: Token[] }>();

		for (const level of granularityLevels) {
			grouped.set(level.id, { positive: [], negative: [] });
		}

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

	/** Count of positive polarity tokens */
	const positiveCount = $derived(tokens.filter((t) => t.polarity === 'positive').length);
	/** Count of negative polarity tokens */
	const negativeCount = $derived(tokens.filter((t) => t.polarity === 'negative').length);

	/**
	 * Handles batch token creation from TokenInput.
	 * Delegates to parent callback with personaId attached.
	 * @param data - Token creation data including granularity, polarity, contents, and weight
	 */
	async function handleAddTokens(data: {
		granularityId: string;
		polarity: TokenPolarity;
		contents: string;
		weight: number;
	}) {
		await onCreateToken?.({
			personaId,
			...data
		});
	}

	/**
	 * Opens the edit modal for a specific token.
	 * @param token - The token to edit
	 */
	function handleEditToken(token: Token) {
		editingToken = token;
	}

	/**
	 * Saves token changes and closes the edit modal.
	 * @param id - Token ID to update
	 * @param data - Updated token fields
	 */
	async function handleSaveToken(id: string, data: UpdateTokenRequest) {
		await onUpdateToken?.(id, data);
		editingToken = null;
	}

	/**
	 * Initiates token deletion by showing confirmation dialog.
	 * @param token - The token to delete
	 */
	function handleDeleteToken(token: Token) {
		tokenToDelete = token;
		showDeleteConfirm = true;
	}

	/**
	 * Executes token deletion after user confirmation.
	 * Resets dialog state after completion.
	 */
	async function confirmDelete() {
		if (tokenToDelete) {
			await onDeleteToken?.(tokenToDelete.id);
			showDeleteConfirm = false;
			tokenToDelete = null;
		}
	}
</script>

<Card>
	<div class="mb-4 flex items-center justify-between">
		<h2 class="text-lg font-semibold text-base-content">Tokens</h2>
		<div class="text-sm text-base-content/60">
			<span class="text-success">+{positiveCount}</span>
			<span class="mx-1">/</span>
			<span class="text-error">-{negativeCount}</span>
		</div>
	</div>

	{#if !isReadOnly}
		<div class="mb-6">
			<TokenInput {granularityLevels} onAdd={handleAddTokens} {isLoading} />
		</div>
	{/if}

	{#if granularityLevels.length === 0}
		<p class="py-8 text-center text-base-content/60">
			Unable to load granularity levels. Please restart the application.
		</p>
	{:else}
		<div class="space-y-4">
			{#each granularityLevels as level (level.id)}
				{@const group = tokensByGranularity().get(level.id)}
				<TokenSection
					granularity={level}
					positiveTokens={group?.positive ?? []}
					negativeTokens={group?.negative ?? []}
					{isReadOnly}
					onEditToken={handleEditToken}
					onDeleteToken={handleDeleteToken}
				/>
			{/each}
		</div>
	{/if}
</Card>

{#if !isReadOnly && editingToken}
	<TokenEditModal
		token={editingToken}
		{granularityLevels}
		{isLoading}
		onSave={handleSaveToken}
		onCancel={() => (editingToken = null)}
	/>
{/if}

{#if !isReadOnly && tokenToDelete}
	<ConfirmDialog
		open={showDeleteConfirm}
		title="Delete Token"
		confirmText="Delete"
		onconfirm={confirmDelete}
		oncancel={() => {
			showDeleteConfirm = false;
			tokenToDelete = null;
		}}
	>
		<p>Are you sure you want to delete the token "<strong>{tokenToDelete.content}</strong>"?</p>
	</ConfirmDialog>
{/if}
