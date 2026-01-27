<!--
@component
TokenCard - Displays a single token with edit/delete actions.

Shows token content with granularity-based background color and optional weight modifier.
Includes a drag handle and edit/delete buttons when in edit mode.
-->
<script lang="ts">
	import type { Token, GranularityLevel } from '$lib/types';

	/**
	 * Component props for TokenCard.
	 * @property token - The token data to display
	 * @property granularityLevels - Available granularity levels for color lookup
	 * @property isReadOnly - Hides action buttons and drag handle when true
	 * @property onEdit - Callback when edit button is clicked
	 * @property onDelete - Callback when delete button is clicked
	 */
	interface Props {
		token: Token;
		granularityLevels: GranularityLevel[];
		isReadOnly?: boolean;
		onEdit?: (token: Token) => void;
		onDelete?: (token: Token) => void;
	}

	let { token, granularityLevels, isReadOnly = false, onEdit, onDelete }: Props = $props();

	/**
	 * Formatted weight display string.
	 * Only shows weight if different from default (1.0).
	 */
	const weightDisplay = $derived(token.weight !== 1.0 ? `(${token.weight.toFixed(1)})` : '');

	/** Get the color for this token's granularity level */
	const granularityColor = $derived(
		granularityLevels.find((l) => l.id === token.granularity_id)?.color ?? 'base'
	);

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
	class="group flex items-center gap-2 border border-base-300 bg-{granularityColor} text-{granularityColor}-content px-3 py-2 transition-colors hover:border-base-content/30"
>
	<!-- Drag handle (only visible in edit mode) -->
	{#if !isReadOnly}
		<div class="cursor-grab" title="Drag to reorder">
			<svg class="h-4 w-4" fill="currentColor" viewBox="0 0 24 24">
				<circle cx="9" cy="6" r="1.5" />
				<circle cx="15" cy="6" r="1.5" />
				<circle cx="9" cy="12" r="1.5" />
				<circle cx="15" cy="12" r="1.5" />
				<circle cx="9" cy="18" r="1.5" />
				<circle cx="15" cy="18" r="1.5" />
			</svg>
		</div>
	{/if}

	<span class="text-sm select-text">
		{token.content}
		{#if weightDisplay}
			<span class="ml-1 opacity-60">{weightDisplay}</span>
		{/if}
	</span>

	{#if !isReadOnly}
		<div class="flex gap-1">
			<button type="button" class="p-1" onclick={handleEdit} title="Edit token">
				<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
					/>
				</svg>
			</button>
			<button type="button" class="p-1" onclick={handleDelete} title="Delete token">
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
