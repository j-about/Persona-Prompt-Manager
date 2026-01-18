<!--
@component
Personas List Page - Displays all personas with filtering capabilities.

Shows a searchable, filterable list of personas with actions for viewing,
editing, duplicating, and deleting. Loads personas from the store on mount.

@route /personas
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { Button, ConfirmDialog } from '$lib/components/ui';
	import { PersonaList, PersonaFilterBar } from '$lib/components/persona';
	import { personaStore } from '$lib/stores';
	import type { Persona } from '$lib/types';

	/** Controls visibility of the delete confirmation dialog */
	let showDeleteConfirm = $state(false);
	/** Persona pending deletion confirmation */
	let personaToDelete = $state<Persona | null>(null);

	/** Current search text filter */
	let searchQuery = $state('');
	/** Currently selected tags for filtering */
	let selectedTags = $state<string[]>([]);

	/** Unique sorted list of all tags across all personas */
	const allTags = $derived([...new Set(personaStore.personas.flatMap((p) => p.tags))].sort());

	/**
	 * Personas filtered by search query and selected tags.
	 * Matches personas where name contains search text (case-insensitive)
	 * AND has at least one selected tag (if any tags selected).
	 */
	const filteredPersonas = $derived(
		personaStore.sortedPersonas.filter((persona) => {
			const matchesSearch =
				searchQuery.trim() === '' ||
				persona.name.toLowerCase().includes(searchQuery.toLowerCase().trim());
			const matchesTags =
				selectedTags.length === 0 || selectedTags.some((tag) => persona.tags.includes(tag));
			return matchesSearch && matchesTags;
		})
	);

	/** True if any filter is active (search or tags) */
	const hasActiveFilters = $derived(searchQuery.trim() !== '' || selectedTags.length > 0);

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
	 * Toggles a tag in the selected tags filter.
	 * @param tag - Tag to toggle
	 */
	function handleTagToggle(tag: string) {
		if (selectedTags.includes(tag)) {
			selectedTags = selectedTags.filter((t) => t !== tag);
		} else {
			selectedTags = [...selectedTags, tag];
		}
	}

	/** Resets all filters to default state */
	function clearFilters() {
		searchQuery = '';
		selectedTags = [];
	}
</script>

<div>
	<div class="mb-6 flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold text-base-content">Personas</h1>
			{#if personaStore.count > 0}
				<p class="mt-1 text-sm text-base-content/60">
					{#if hasActiveFilters}
						{filteredPersonas.length} of {personaStore.count} persona{personaStore.count === 1
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
			availableTags={allTags}
			onSearchChange={handleSearchChange}
			onTagToggle={handleTagToggle}
			onClearFilters={clearFilters}
		/>
	{/if}

	<PersonaList
		personas={filteredPersonas}
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
