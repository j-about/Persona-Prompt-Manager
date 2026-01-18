<!--
@component
TokenInput - Form for adding new tokens with batch support.

Allows entering single or comma-separated tokens with configurable
polarity, granularity level, and optional weight modifier. Supports
batch creation by parsing comma-delimited input.
-->
<script lang="ts">
	import type { TokenPolarity, GranularityLevel } from '$lib/types';
	import { Button } from '$lib/components/ui';

	/**
	 * Component props for TokenInput.
	 * @property granularityLevels - Available granularity options for the dropdown
	 * @property onAdd - Callback invoked with token data on form submission
	 * @property isLoading - Disables form inputs during async operations
	 */
	interface Props {
		granularityLevels: GranularityLevel[];
		onAdd: (data: {
			granularityId: string;
			polarity: TokenPolarity;
			contents: string;
			weight: number;
		}) => void;
		isLoading?: boolean;
	}

	let { granularityLevels, onAdd, isLoading = false }: Props = $props();

	/** Raw token content input (may contain commas for batch) */
	let content = $state('');
	/** Selected granularity level ID */
	let granularityId = $state('');
	/** Token polarity: positive or negative */
	let polarity = $state<TokenPolarity>('positive');
	/** Token weight multiplier (default 1.0) */
	let weight = $state(1.0);
	/** Controls visibility of the weight input */
	let showAdvanced = $state(false);

	/**
	 * Initializes granularity selection to first available level.
	 * Runs when granularityLevels changes and no selection exists.
	 */
	$effect(() => {
		if (granularityLevels.length > 0 && !granularityId) {
			granularityId = granularityLevels[0].id;
		}
	});

	/** Form is valid when content and granularity are provided */
	const isValid = $derived(content.trim().length > 0 && granularityId.length > 0);

	/**
	 * Handles form submission by invoking onAdd callback.
	 * Resets content and weight after successful submission.
	 * @param e - Form submit event
	 */
	function handleSubmit(e: Event) {
		e.preventDefault();
		if (!isValid || isLoading) return;

		onAdd({
			granularityId,
			polarity,
			contents: content.trim(),
			weight
		});

		content = '';
		weight = 1.0;
	}

	/** Toggles polarity between positive and negative */
	function togglePolarity() {
		polarity = polarity === 'positive' ? 'negative' : 'positive';
	}
</script>

<form onsubmit={handleSubmit} class="space-y-3">
	<div class="flex gap-2">
		<button
			type="button"
			class="btn btn-square btn-sm {polarity === 'positive'
				? 'btn-soft btn-success'
				: 'btn-soft btn-error'}"
			onclick={togglePolarity}
			title="Toggle polarity"
		>
			{polarity === 'positive' ? '+' : '-'}
		</button>

		<input
			type="text"
			bind:value={content}
			placeholder="Add tokens (comma-separated for batch)"
			class="input-bordered input input-sm flex-1"
			disabled={isLoading}
		/>

		<select
			bind:value={granularityId}
			class="select-bordered select select-sm"
			disabled={isLoading}
		>
			{#each granularityLevels as level (level.id)}
				<option value={level.id}>{level.name}</option>
			{/each}
		</select>

		<Button type="submit" size="sm" disabled={!isValid || isLoading}>Add</Button>
	</div>

	<div class="flex items-center gap-4">
		<button
			type="button"
			class="btn text-base-content/60 btn-ghost btn-xs"
			onclick={() => (showAdvanced = !showAdvanced)}
		>
			{showAdvanced ? 'Hide' : 'Show'} weight option
		</button>

		{#if showAdvanced}
			<div class="flex items-center gap-2">
				<label for="weight" class="text-xs text-base-content/60">Weight:</label>
				<input
					type="number"
					id="weight"
					bind:value={weight}
					min="0.1"
					max="2.0"
					step="0.1"
					class="input-bordered input input-sm w-20"
					disabled={isLoading}
				/>
			</div>
		{/if}
	</div>

	<p class="text-xs text-base-content/60">
		Tip: Enter multiple tokens separated by commas to add them all at once
	</p>
</form>
