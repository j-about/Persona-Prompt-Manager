<!--
@component
TokenCard - Displays a single token with edit/delete actions.

Shows token content, polarity badge, and optional weight modifier.
Edit and delete buttons appear on hover when not in read-only mode.
-->
<script lang="ts">
	import type { Token } from '$lib/types';
	import { Badge } from '$lib/components/ui';

	/**
	 * Component props for TokenCard.
	 * @property token - The token data to display
	 * @property isReadOnly - Hides action buttons when true
	 * @property onEdit - Callback when edit button is clicked
	 * @property onDelete - Callback when delete button is clicked
	 */
	interface Props {
		token: Token;
		isReadOnly?: boolean;
		onEdit?: (token: Token) => void;
		onDelete?: (token: Token) => void;
	}

	let { token, isReadOnly = false, onEdit, onDelete }: Props = $props();

	/**
	 * Formatted weight display string.
	 * Only shows weight if different from default (1.0).
	 */
	const weightDisplay = $derived(token.weight !== 1.0 ? `(${token.weight.toFixed(1)})` : '');

	/** Triggers edit callback with current token */
	function handleEdit() {
		onEdit?.(token);
	}

	/** Triggers delete callback with current token */
	function handleDelete() {
		onDelete?.(token);
	}
</script>

<div
	class="group flex items-center gap-2 rounded-lg border border-base-300 bg-base-100 px-3 py-2 transition-colors hover:border-base-content/30"
>
	<Badge variant={token.polarity === 'positive' ? 'positive' : 'negative'} size="sm">
		{token.polarity === 'positive' ? '+' : '-'}
	</Badge>

	<span class="flex-1 text-sm text-base-content">
		{token.content}
		{#if weightDisplay}
			<span class="ml-1 text-base-content/60">{weightDisplay}</span>
		{/if}
	</span>

	{#if !isReadOnly}
		<div class="flex gap-1 opacity-0 transition-opacity group-hover:opacity-100">
			<button
				type="button"
				class="p-1 text-base-content/50 hover:text-base-content"
				onclick={handleEdit}
				title="Edit token"
			>
				<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
					/>
				</svg>
			</button>
			<button
				type="button"
				class="p-1 text-base-content/50 hover:text-error"
				onclick={handleDelete}
				title="Delete token"
			>
				<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
					/>
				</svg>
			</button>
		</div>
	{/if}
</div>
