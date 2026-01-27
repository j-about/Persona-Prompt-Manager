<!--
@component
TokenManager - Orchestrates token CRUD operations within a persona context.

Displays all tokens in a flat list ordered by user-defined display_order.
Supports drag-and-drop reordering when in edit mode. Shows a color-coded
legend to identify granularity levels.
-->
<script lang="ts">
	import { flip } from 'svelte/animate';
	import { dndzone } from 'svelte-dnd-action';
	import type {
		Token,
		GranularityLevel,
		TokenPolarity,
		UpdateTokenRequest,
		TokenOrderUpdate,
		TokenCount
	} from '$lib/types';
	import { Card, ConfirmDialog, TokenCountBadge } from '$lib/components/ui';
	import { countTokens } from '$lib/services/tokenizer';
	import TokenInput from './TokenInput.svelte';
	import TokenCard from './TokenCard.svelte';
	import TokenLegend from './TokenLegend.svelte';
	import TokenEditModal from './TokenEditModal.svelte';

	/**
	 * Component props for TokenManager.
	 * @property personaId - The ID of the persona owning these tokens
	 * @property tokens - Array of tokens to display and manage
	 * @property granularityLevels - Available granularity categories for labeling
	 * @property isLoading - Disables interactions during async operations
	 * @property isReadOnly - Hides editing controls when true
	 * @property onCreateToken - Callback for creating new tokens
	 * @property onUpdateToken - Callback for updating existing tokens
	 * @property onDeleteToken - Callback for deleting tokens
	 * @property onReorderTokens - Callback for reordering tokens via drag-and-drop
	 * @property modelId - Optional model ID for tokenizer selection
	 */
	interface Props {
		personaId: string;
		tokens: Token[];
		granularityLevels: GranularityLevel[];
		isLoading?: boolean;
		isReadOnly?: boolean;
		modelId?: string;
		onCreateToken?: (data: {
			personaId: string;
			granularityId: string;
			polarity: TokenPolarity;
			contents: string;
			weight: number;
		}) => Promise<void>;
		onUpdateToken?: (id: string, data: UpdateTokenRequest) => Promise<void>;
		onDeleteToken?: (id: string) => Promise<void>;
		onReorderTokens?: (orders: TokenOrderUpdate[]) => void;
	}

	let {
		personaId,
		tokens,
		granularityLevels,
		isLoading = false,
		isReadOnly = false,
		modelId,
		onCreateToken,
		onUpdateToken,
		onDeleteToken,
		onReorderTokens
	}: Props = $props();

	/** Token currently being edited in the modal, null if no edit in progress */
	let editingToken = $state<Token | null>(null);
	/** Controls visibility of the delete confirmation dialog */
	let showDeleteConfirm = $state(false);
	/** Token pending deletion confirmation */
	let tokenToDelete = $state<Token | null>(null);
	/** Whether to include token weights in counting (view mode only) */
	let includeWeights = $state(true);

	/** Animation duration for drag-and-drop flip effect */
	const flipDurationMs = 200;

	/**
	 * Positive tokens sorted by display_order.
	 */
	const positiveTokens = $derived(
		tokens
			.filter((t) => t.polarity === 'positive')
			.sort((a, b) => a.display_order - b.display_order)
	);

	/**
	 * Negative tokens sorted by display_order.
	 */
	const negativeTokens = $derived(
		tokens
			.filter((t) => t.polarity === 'negative')
			.sort((a, b) => a.display_order - b.display_order)
	);

	/**
	 * Mutable items arrays for drag-and-drop.
	 * Separate arrays for each polarity to prevent mixing.
	 * We use $state + $effect instead of $derived because svelte-dnd-action
	 * requires direct mutation of the arrays during drag operations.
	 */
	// eslint-disable-next-line svelte/prefer-writable-derived -- svelte-dnd-action requires mutable array
	let dndPositiveItems = $state<Token[]>([]);
	// eslint-disable-next-line svelte/prefer-writable-derived -- svelte-dnd-action requires mutable array
	let dndNegativeItems = $state<Token[]>([]);

	// Sync dndPositiveItems when positiveTokens changes
	$effect(() => {
		dndPositiveItems = [...positiveTokens];
	});

	// Sync dndNegativeItems when negativeTokens changes
	$effect(() => {
		dndNegativeItems = [...negativeTokens];
	});

	/** Count of positive polarity tokens */
	const positiveCount = $derived(positiveTokens.length);
	/** Count of negative polarity tokens */
	const negativeCount = $derived(negativeTokens.length);

	/** Tokenizer count for positive prompt */
	let positiveTokenCount = $state<TokenCount | null>(null);
	/** Tokenizer count for negative prompt */
	let negativeTokenCount = $state<TokenCount | null>(null);
	/** Token counting in progress */
	let isCountingTokens = $state(false);

	/**
	 * Formats tokens as comma-separated prompt string with optional weights.
	 * Matches the format used in prompt composition.
	 */
	function formatTokensForCounting(tokenList: Token[]): string {
		return tokenList
			.map((t) => (includeWeights && t.weight !== 1.0 ? `(${t.content}:${t.weight})` : t.content))
			.join(', ');
	}

	// Reactively count tokens when tokens, modelId, or includeWeights change
	$effect(() => {
		const positiveText = formatTokensForCounting(positiveTokens);
		const negativeText = formatTokensForCounting(negativeTokens);

		isCountingTokens = true;
		Promise.all([countTokens(positiveText, modelId), countTokens(negativeText, modelId)])
			.then(([pos, neg]) => {
				positiveTokenCount = pos;
				negativeTokenCount = neg;
			})
			.finally(() => {
				isCountingTokens = false;
			});
	});

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

	/**
	 * Handles drag-and-drop consider events for positive tokens.
	 */
	function handlePositiveDndConsider(e: CustomEvent<{ items: Token[] }>) {
		dndPositiveItems = e.detail.items;
	}

	/**
	 * Handles drag-and-drop finalize events for positive tokens.
	 */
	function handlePositiveDndFinalize(e: CustomEvent<{ items: Token[] }>) {
		dndPositiveItems = e.detail.items;

		// Compute new display_order for positive tokens based on position
		const orders: TokenOrderUpdate[] = dndPositiveItems.map((token, index) => ({
			token_id: token.id,
			display_order: index
		}));

		onReorderTokens?.(orders);
	}

	/**
	 * Handles drag-and-drop consider events for negative tokens.
	 */
	function handleNegativeDndConsider(e: CustomEvent<{ items: Token[] }>) {
		dndNegativeItems = e.detail.items;
	}

	/**
	 * Handles drag-and-drop finalize events for negative tokens.
	 */
	function handleNegativeDndFinalize(e: CustomEvent<{ items: Token[] }>) {
		dndNegativeItems = e.detail.items;

		// Compute new display_order for negative tokens based on position
		const orders: TokenOrderUpdate[] = dndNegativeItems.map((token, index) => ({
			token_id: token.id,
			display_order: index
		}));

		onReorderTokens?.(orders);
	}
</script>

<Card>
	<div class="mb-4">
		<h2 class="text-lg font-semibold text-base-content">Tokens</h2>
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
		<!-- Granularity color legend -->
		<div class="mb-4">
			<TokenLegend {granularityLevels} />
		</div>

		{#if isReadOnly}
			<div class="mb-4">
				<label class="label cursor-pointer justify-start gap-2">
					<input
						type="checkbox"
						class="checkbox checkbox-sm checkbox-warning"
						bind:checked={includeWeights}
					/>
					<span class="text-sm text-base-content">Include token weights</span>
				</label>
			</div>
		{/if}

		{#if tokens.length === 0}
			<p class="py-8 text-center text-base-content/60">
				{isReadOnly
					? 'No tokens defined for this persona.'
					: 'No tokens yet. Use the form above to add tokens.'}
			</p>
		{:else}
			<div class="space-y-6">
				<!-- Positive Tokens Section -->
				<div>
					<div class="mb-2 flex items-center gap-2">
						<span class="inline-block h-2 w-2 rounded-full bg-success"></span>
						<h3 class="text-sm font-medium text-base-content">Positive</h3>
						{#if isReadOnly}
							<TokenCountBadge tokenCount={positiveTokenCount} isLoading={isCountingTokens} />
						{/if}
					</div>
					{#if isReadOnly}
						<div
							class="flex min-h-[40px] flex-wrap gap-2 border border-dashed border-success/30 p-2"
						>
							{#if positiveCount === 0}
								<p class="w-full py-2 text-center text-sm text-base-content/50">
									No positive tokens
								</p>
							{:else}
								{#each positiveTokens as token (token.id)}
									<TokenCard {token} {granularityLevels} {isReadOnly} />
								{/each}
							{/if}
						</div>
					{:else}
						<div
							class="flex min-h-[40px] flex-wrap gap-2 border border-dashed border-success/30 p-2"
							use:dndzone={{ items: dndPositiveItems, flipDurationMs, dragDisabled: isLoading }}
							onconsider={handlePositiveDndConsider}
							onfinalize={handlePositiveDndFinalize}
						>
							{#if positiveCount === 0}
								<p class="w-full py-2 text-center text-sm text-base-content/50">
									No positive tokens
								</p>
							{/if}
							{#each dndPositiveItems as token (token.id)}
								<div animate:flip={{ duration: flipDurationMs }}>
									<TokenCard
										{token}
										{granularityLevels}
										{isReadOnly}
										onEdit={handleEditToken}
										onDelete={handleDeleteToken}
									/>
								</div>
							{/each}
						</div>
					{/if}
				</div>

				<!-- Negative Tokens Section -->
				<div>
					<div class="mb-2 flex items-center gap-2">
						<span class="inline-block h-2 w-2 rounded-full bg-error"></span>
						<h3 class="text-sm font-medium text-base-content">Negative</h3>
						{#if isReadOnly}
							<TokenCountBadge tokenCount={negativeTokenCount} isLoading={isCountingTokens} />
						{/if}
					</div>
					{#if isReadOnly}
						<div class="flex min-h-[40px] flex-wrap gap-2 border border-dashed border-error/30 p-2">
							{#if negativeCount === 0}
								<p class="w-full py-2 text-center text-sm text-base-content/50">
									No negative tokens
								</p>
							{:else}
								{#each negativeTokens as token (token.id)}
									<TokenCard {token} {granularityLevels} {isReadOnly} />
								{/each}
							{/if}
						</div>
					{:else}
						<div
							class="flex min-h-[40px] flex-wrap gap-2 border border-dashed border-error/30 p-2"
							use:dndzone={{ items: dndNegativeItems, flipDurationMs, dragDisabled: isLoading }}
							onconsider={handleNegativeDndConsider}
							onfinalize={handleNegativeDndFinalize}
						>
							{#if negativeCount === 0}
								<p class="w-full py-2 text-center text-sm text-base-content/50">
									No negative tokens
								</p>
							{/if}
							{#each dndNegativeItems as token (token.id)}
								<div animate:flip={{ duration: flipDurationMs }}>
									<TokenCard
										{token}
										{granularityLevels}
										{isReadOnly}
										onEdit={handleEditToken}
										onDelete={handleDeleteToken}
									/>
								</div>
							{/each}
						</div>
					{/if}
				</div>
			</div>
		{/if}
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
