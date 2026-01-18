<!--
@component
TokenSection - Collapsible section displaying tokens for a single granularity level.

Organizes tokens into positive and negative groups within an expandable panel.
Shows token count in the header and allows toggling visibility.
-->
<script lang="ts">
	import type { Token, GranularityLevel } from '$lib/types';
	import TokenCard from './TokenCard.svelte';

	/**
	 * Component props for TokenSection.
	 * @property granularity - The granularity level metadata (id, name)
	 * @property positiveTokens - Tokens with positive polarity for this level
	 * @property negativeTokens - Tokens with negative polarity for this level
	 * @property isReadOnly - Passed to child TokenCards to hide actions
	 * @property onEditToken - Callback when a token edit is requested
	 * @property onDeleteToken - Callback when a token delete is requested
	 */
	interface Props {
		granularity: GranularityLevel;
		positiveTokens: Token[];
		negativeTokens: Token[];
		isReadOnly?: boolean;
		onEditToken?: (token: Token) => void;
		onDeleteToken?: (token: Token) => void;
	}

	let {
		granularity,
		positiveTokens,
		negativeTokens,
		isReadOnly = false,
		onEditToken,
		onDeleteToken
	}: Props = $props();

	/** Combined count of positive and negative tokens */
	const totalCount = $derived(positiveTokens.length + negativeTokens.length);
	/** Controls section collapse state, expanded by default */
	let isCollapsed = $state(false);
</script>

<div class="overflow-hidden rounded-lg border border-base-300">
	<button
		type="button"
		class="flex w-full items-center justify-between bg-base-200 px-4 py-3 transition-colors hover:bg-base-300"
		onclick={() => (isCollapsed = !isCollapsed)}
	>
		<div class="flex items-center gap-2">
			<svg
				class="h-4 w-4 text-base-content/60 transition-transform {isCollapsed ? '' : 'rotate-90'}"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
			</svg>
			<span class="font-medium text-base-content">{granularity.name}</span>
		</div>
		<span class="text-sm text-base-content/60">
			{totalCount} token{totalCount === 1 ? '' : 's'}
		</span>
	</button>

	{#if !isCollapsed}
		<div class="space-y-4 p-4">
			{#if positiveTokens.length > 0}
				<div>
					<h4 class="mb-2 text-xs font-medium text-success uppercase">
						Positive ({positiveTokens.length})
					</h4>
					<div class="flex flex-wrap gap-2">
						{#each positiveTokens as token (token.id)}
							<TokenCard {token} {isReadOnly} onEdit={onEditToken} onDelete={onDeleteToken} />
						{/each}
					</div>
				</div>
			{/if}

			{#if negativeTokens.length > 0}
				<div>
					<h4 class="mb-2 text-xs font-medium text-error uppercase">
						Negative ({negativeTokens.length})
					</h4>
					<div class="flex flex-wrap gap-2">
						{#each negativeTokens as token (token.id)}
							<TokenCard {token} {isReadOnly} onEdit={onEditToken} onDelete={onDeleteToken} />
						{/each}
					</div>
				</div>
			{/if}

			{#if totalCount === 0}
				<p class="py-4 text-center text-sm text-base-content/60">No tokens for this level yet</p>
			{/if}
		</div>
	{/if}
</div>
