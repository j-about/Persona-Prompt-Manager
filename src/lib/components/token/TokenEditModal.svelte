<!--
@component
TokenEditModal - Modal dialog for editing an existing token.

Provides form fields to modify token content, granularity, polarity, and weight.
Uses untrack() to capture initial values as a snapshot, preventing form state
from updating if the source token prop changes during editing.
-->
<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import type { Token, GranularityLevel, TokenPolarity, UpdateTokenRequest } from '$lib/types';
	import { Button } from '$lib/components/ui';

	/**
	 * Component props for TokenEditModal.
	 * @property token - The token to edit (used to initialize form state)
	 * @property granularityLevels - Available granularity options for the dropdown
	 * @property isLoading - Disables form inputs and buttons during save
	 * @property onSave - Callback with token ID and changed fields
	 * @property onCancel - Callback when modal is closed without saving
	 */
	interface Props {
		token: Token;
		granularityLevels: GranularityLevel[];
		isLoading?: boolean;
		onSave: (id: string, data: UpdateTokenRequest) => void;
		onCancel: () => void;
	}

	let { token, granularityLevels, isLoading = false, onSave, onCancel }: Props = $props();

	/** Reference to the native dialog element for showModal/close control */
	let dialogRef: HTMLDialogElement;

	/**
	 * Form state initialized from token using untrack() to prevent reactive updates.
	 * This creates a snapshot of the original values for comparison and editing.
	 */
	let content = $state(untrack(() => token.content));
	let granularityId = $state(untrack(() => token.granularity_id));
	let polarity = $state<TokenPolarity>(untrack(() => token.polarity));
	let weight = $state(untrack(() => token.weight));

	/** Form is valid when content is non-empty */
	const isValid = $derived(content.trim().length > 0);

	/**
	 * Tracks whether any field has been modified from original values.
	 * Used to enable/disable the save button.
	 */
	const hasChanges = $derived(
		content !== token.content ||
			granularityId !== token.granularity_id ||
			polarity !== token.polarity ||
			weight !== token.weight
	);

	/** Opens the dialog as a modal when component mounts */
	onMount(() => {
		dialogRef?.showModal();
	});

	/** Closes the dialog and triggers the cancel callback */
	function handleClose() {
		dialogRef?.close();
		onCancel();
	}

	/**
	 * Submits only the changed fields to minimize update payload.
	 * @param e - Form submit event
	 */
	function handleSubmit(e: Event) {
		e.preventDefault();
		if (!isValid || isLoading || !hasChanges) return;

		const updates: UpdateTokenRequest = {};
		if (content !== token.content) updates.content = content.trim();
		if (granularityId !== token.granularity_id) updates.granularity_id = granularityId;
		if (polarity !== token.polarity) updates.polarity = polarity;
		if (weight !== token.weight) updates.weight = weight;

		onSave(token.id, updates);
	}
</script>

<dialog bind:this={dialogRef} class="modal" onclose={onCancel}>
	<div class="modal-box">
		<h3 class="text-lg font-bold">Edit Token</h3>

		<form onsubmit={handleSubmit} class="mt-4 space-y-4">
			<div class="grid grid-cols-2 gap-4">
				<div>
					<label for="granularity" class="label">
						<span class="label-text">Granularity</span>
					</label>
					<select
						id="granularity"
						bind:value={granularityId}
						class="select-bordered select w-full"
						disabled={isLoading}
					>
						{#each granularityLevels as level (level.id)}
							<option value={level.id}>{level.name}</option>
						{/each}
					</select>
				</div>

				<div>
					<label for="polarity" class="label">
						<span class="label-text">Polarity</span>
					</label>
					<select
						id="polarity"
						bind:value={polarity}
						class="select-bordered select w-full"
						disabled={isLoading}
					>
						<option value="positive">Positive (+)</option>
						<option value="negative">Negative (-)</option>
					</select>
				</div>
			</div>

			<div>
				<label for="content" class="label">
					<span class="label-text">Content</span>
				</label>
				<input
					type="text"
					id="content"
					bind:value={content}
					class="input-bordered input w-full"
					disabled={isLoading}
				/>
			</div>

			<div>
				<label for="weight" class="label">
					<span class="label-text">Weight</span>
				</label>
				<div class="flex items-center gap-3">
					<input
						type="range"
						id="weight"
						bind:value={weight}
						min="0.1"
						max="2.0"
						step="0.1"
						class="range flex-1 range-primary"
						disabled={isLoading}
					/>
					<span class="w-12 text-right text-sm text-base-content">
						{weight.toFixed(1)}
					</span>
				</div>
			</div>

			<div class="modal-action">
				<Button type="button" variant="secondary" onclick={handleClose} disabled={isLoading}>
					Cancel
				</Button>
				<Button type="submit" disabled={!isValid || !hasChanges || isLoading}>
					{#if isLoading}
						Saving...
					{:else}
						Save Changes
					{/if}
				</Button>
			</div>
		</form>
	</div>
	<form method="dialog" class="modal-backdrop">
		<button onclick={onCancel}>close</button>
	</form>
</dialog>
