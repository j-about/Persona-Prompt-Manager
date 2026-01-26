<!--
@component
Personas List Page - Displays all personas with filtering and sorting capabilities.

Shows a searchable, filterable, sortable list of personas with actions for viewing,
editing, duplicating, and deleting. Loads personas from the store on mount.

@route /personas
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { Button, ConfirmDialog } from '$lib/components/ui';
	import { PersonaList, PersonaFilterBar } from '$lib/components/persona';
	import { personaStore, uiPreferencesStore } from '$lib/stores';
	import type { Persona } from '$lib/types';

	/** Controls visibility of the delete confirmation dialog */
	let showDeleteConfirm = $state(false);
	/** Persona pending deletion confirmation */
	let personaToDelete = $state<Persona | null>(null);

	/** Current search text filter */
	let searchQuery = $state('');

	/** Currently selected tags for filtering (session-persisted via store) */
	const selectedTags = $derived(uiPreferencesStore.personaListTags);
	/** Current sort value (file-persisted via store) */
	const sortValue = $derived(uiPreferencesStore.personaListSort);

	/** Unique sorted list of all tags across all personas */
	const allTags = $derived([...new Set(personaStore.personas.flatMap((p) => p.tags))].sort());

	/**
	 * Tag options with disabled state based on current selection.
	 * A tag is disabled if selecting it would result in zero matching personas.
	 */
	const availableTagOptions = $derived.by(() => {
		return allTags.map((tag) => {
			// No selection: all tags available
			if (selectedTags.length === 0) {
				return { value: tag, disabled: false };
			}
			// Already selected: keep enabled to allow deselection
			if (selectedTags.includes(tag)) {
				return { value: tag, disabled: false };
			}
			// Check if any persona has ALL selected tags AND this tag
			const wouldHaveResults = personaStore.personas.some((persona) => {
				const hasAllSelected = selectedTags.every((t) => persona.tags.includes(t));
				return hasAllSelected && persona.tags.includes(tag);
			});
			return { value: tag, disabled: !wouldHaveResults };
		});
	});

	/**
	 * Personas filtered by search query and selected tags, then sorted.
	 * Matches personas where name contains search text (case-insensitive)
	 * AND has ALL selected tags (if any tags selected).
	 */
	const filteredAndSortedPersonas = $derived.by(() => {
		// Filter
		const filtered = personaStore.personas.filter((persona) => {
			const matchesSearch =
				searchQuery.trim() === '' ||
				persona.name.toLowerCase().includes(searchQuery.toLowerCase().trim());
			const matchesTags =
				selectedTags.length === 0 || selectedTags.every((tag) => persona.tags.includes(tag));
			return matchesSearch && matchesTags;
		});

		// Sort
		const [field, direction] = sortValue.split('-') as [string, 'asc' | 'desc'];
		const sorted = [...filtered].sort((a, b) => {
			let comparison = 0;
			if (field === 'name') {
				comparison = a.name.localeCompare(b.name);
			} else if (field === 'created_at') {
				comparison = new Date(a.created_at).getTime() - new Date(b.created_at).getTime();
			} else if (field === 'updated_at') {
				comparison = new Date(a.updated_at).getTime() - new Date(b.updated_at).getTime();
			}
			return direction === 'desc' ? -comparison : comparison;
		});

		return sorted;
	});

	/** True if any filter is active (search, tags, or non-default sort) */
	const hasActiveFilters = $derived(
		searchQuery.trim() !== '' || selectedTags.length > 0 || sortValue !== 'updated_at-desc'
	);

	/** Loads all personas from the backend on mount */
	onMount(() => {
		personaStore.loadAll();
	});

	/**
	 * Navigates to the persona detail view.
	 * @param persona - The selected persona
	 */
	function handleSelect(persona: Persona) {
		goto(resolve(`/personas/${persona.id}`));
	}

	/**
	 * Navigates to the persona detail view in edit mode.
	 * @param persona - The persona to edit
	 */
	function handleEdit(persona: Persona) {
		goto(resolve(`/personas/${persona.id}?edit=true`));
	}

	/**
	 * Opens the delete confirmation dialog for a persona.
	 * @param persona - The persona to delete
	 */
	function handleDelete(persona: Persona) {
		personaToDelete = persona;
		showDeleteConfirm = true;
	}

	/** Executes persona deletion and resets dialog state */
	async function confirmDelete() {
		if (personaToDelete) {
			await personaStore.remove(personaToDelete.id);
			showDeleteConfirm = false;
			personaToDelete = null;
		}
	}

	/** Closes delete dialog without deleting */
	function cancelDelete() {
		showDeleteConfirm = false;
		personaToDelete = null;
	}

	/**
	 * Creates a duplicate of the persona.
	 * @param persona - The persona to duplicate
	 */
	async function handleDuplicate(persona: Persona) {
		await personaStore.duplicate(persona.id);
	}

	/**
	 * Updates the search query filter.
	 * @param query - New search text
	 */
	function handleSearchChange(query: string) {
		searchQuery = query;
	}

	/**
	 * Updates the selected tags filter (session-persisted).
	 * @param tags - New selected tags
	 */
	function handleTagsChange(tags: string[]) {
		uiPreferencesStore.setPersonaListTags(tags);
	}

	/**
	 * Updates the sort value (file-persisted).
	 * @param value - New sort value
	 */
	function handleSortChange(value: string) {
		uiPreferencesStore.setPersonaListSort(value);
	}
</script>

<div>
	<div class="mb-6 flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold text-base-content">Personas</h1>
			{#if personaStore.count > 0}
				<p class="mt-1 text-sm text-base-content/60">
					{#if hasActiveFilters}
						{filteredAndSortedPersonas.length} of {personaStore.count} persona{personaStore.count ===
						1
							? ''
							: 's'}
					{:else}
						{personaStore.count} persona{personaStore.count === 1 ? '' : 's'}
					{/if}
				</p>
			{/if}
		</div>
		<a href={resolve('/personas/new')}>
			<Button>Create New Persona</Button>
		</a>
	</div>

	{#if personaStore.error}
		<div role="alert" class="mb-4 alert alert-soft alert-error">
			<span>{personaStore.error}</span>
			<button type="button" class="btn btn-ghost btn-xs" onclick={() => personaStore.clearError()}>
				Dismiss
			</button>
		</div>
	{/if}

	{#if personaStore.count > 0}
		<PersonaFilterBar
			{searchQuery}
			{selectedTags}
			availableTags={availableTagOptions}
			{sortValue}
			onSearchChange={handleSearchChange}
			onTagsChange={handleTagsChange}
			onSortChange={handleSortChange}
		/>
	{/if}

	<PersonaList
		personas={filteredAndSortedPersonas}
		isLoading={personaStore.isLoading}
		emptyMessage={hasActiveFilters ? 'No personas match your filters' : undefined}
		onSelect={handleSelect}
		onEdit={handleEdit}
		onDelete={handleDelete}
		onDuplicate={handleDuplicate}
	/>
</div>

{#if personaToDelete}
	<ConfirmDialog
		open={showDeleteConfirm}
		title="Delete Persona"
		confirmText="Delete"
		onconfirm={confirmDelete}
		oncancel={cancelDelete}
	>
		<p>
			Are you sure you want to delete <strong>{personaToDelete.name}</strong>? This will also delete
			all associated tokens. This action cannot be undone.
		</p>
	</ConfirmDialog>
{/if}
