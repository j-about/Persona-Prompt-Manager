<script lang="ts">
	import { Button } from '$lib/components/ui';

	interface Props {
		searchQuery: string;
		selectedTags: string[];
		availableTags: string[];
		onSearchChange: (query: string) => void;
		onTagToggle: (tag: string) => void;
		onClearFilters: () => void;
	}

	let {
		searchQuery,
		selectedTags,
		availableTags,
		onSearchChange,
		onTagToggle,
		onClearFilters
	}: Props = $props();

	const hasActiveFilters = $derived(searchQuery.trim() !== '' || selectedTags.length > 0);
</script>

<div class="mb-6 space-y-3">
	<div class="relative">
		<svg
			class="absolute top-1/2 left-3 h-5 w-5 -translate-y-1/2 text-base-content/50"
			fill="none"
			viewBox="0 0 24 24"
			stroke="currentColor"
		>
			<path
				stroke-linecap="round"
				stroke-linejoin="round"
				stroke-width="2"
				d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
			/>
		</svg>
		<input
			type="text"
			placeholder="Search personas by name..."
			class="input-bordered input w-full pl-10"
			value={searchQuery}
			oninput={(e) => onSearchChange(e.currentTarget.value)}
		/>
	</div>

	{#if availableTags.length > 0}
		<div class="flex flex-wrap items-center gap-2">
			<span class="text-sm text-base-content/60">Filter by tags:</span>
			{#each availableTags as tag (tag)}
				<button
					type="button"
					class="badge cursor-pointer badge-lg transition-colors {selectedTags.includes(tag)
						? 'badge-primary'
						: 'hover:badge-primary/30 badge-ghost'}"
					onclick={() => onTagToggle(tag)}
				>
					{tag}
				</button>
			{/each}
			{#if hasActiveFilters}
				<Button variant="ghost" size="sm" class="ml-auto" onclick={onClearFilters}
					>Clear filters</Button
				>
			{/if}
		</div>
	{:else if hasActiveFilters}
		<div class="flex justify-end">
			<Button variant="ghost" size="sm" class="ml-auto" onclick={onClearFilters}
				>Clear filters</Button
			>
		</div>
	{/if}
</div>
