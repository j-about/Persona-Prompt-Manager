<!--
@component
PersonaFilterBar - Search, sort, and tag filter controls for persona list.

Provides a search input for filtering by name, a sort dropdown for ordering,
and a multiselect dropdown for filtering by tags. Controls are right-aligned.
-->
<script lang="ts">
	import { MultiSelect } from '$lib/components/ui';

	/** Option with value and optional disabled state */
	interface SelectOption {
		value: string;
		disabled?: boolean;
	}

	/**
	 * @property searchQuery - Current search text (bindable)
	 * @property selectedTags - Currently selected tag filters
	 * @property availableTags - Tag options with disabled state for incompatible combinations
	 * @property sortValue - Current sort value (e.g., 'name-asc')
	 * @property onSearchChange - Callback when search text changes
	 * @property onTagsChange - Callback when tag selection changes
	 * @property onSortChange - Callback when sort selection changes
	 */
	interface Props {
		searchQuery: string;
		selectedTags: string[];
		availableTags: SelectOption[];
		sortValue: string;
		onSearchChange: (query: string) => void;
		onTagsChange: (tags: string[]) => void;
		onSortChange: (value: string) => void;
	}

	let {
		searchQuery,
		selectedTags,
		availableTags,
		sortValue,
		onSearchChange,
		onTagsChange,
		onSortChange
	}: Props = $props();

	/** Sort options with display labels */
	const sortOptions = [
		{ value: 'name-asc', label: 'Name (A-Z)' },
		{ value: 'name-desc', label: 'Name (Z-A)' },
		{ value: 'created_at-desc', label: 'Created (Newest)' },
		{ value: 'created_at-asc', label: 'Created (Oldest)' },
		{ value: 'updated_at-desc', label: 'Updated (Newest)' },
		{ value: 'updated_at-asc', label: 'Updated (Oldest)' }
	];
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

	<div class="flex flex-wrap items-center justify-end gap-2">
		{#if availableTags.length > 0}
			<MultiSelect
				options={availableTags}
				selected={selectedTags}
				placeholder="Filter by tags"
				onchange={onTagsChange}
			/>
		{/if}

		<select
			class="select-bordered select w-auto select-sm"
			value={sortValue}
			onchange={(e) => onSortChange(e.currentTarget.value)}
		>
			{#each sortOptions as option (option.value)}
				<option value={option.value}>{option.label}</option>
			{/each}
		</select>
	</div>
</div>
