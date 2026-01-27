<!--
@component
TokenLegend - Displays a color legend for granularity levels.

Shows all granularity levels with their corresponding background colors
to help users understand the token categorization system.
-->
<script lang="ts">
	import type { GranularityLevel } from '$lib/types';

	/**
	 * @property granularityLevels - Available granularity level definitions
	 */
	interface Props {
		granularityLevels: GranularityLevel[];
	}

	let { granularityLevels }: Props = $props();

	/** Granularity levels sorted by display order */
	const sortedLevels = $derived(
		[...granularityLevels].sort((a, b) => a.display_order - b.display_order)
	);
</script>

<div class="flex flex-wrap items-center gap-2 bg-base-200 p-2">
	<span class="text-xs text-base-content/60">Granularity Levels:</span>
	{#each sortedLevels as level (level.id)}
		<div class="badge badge-{level.color}">
			{level.name}
		</div>
	{/each}
</div>
