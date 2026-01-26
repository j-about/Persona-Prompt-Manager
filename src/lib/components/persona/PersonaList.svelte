<!--
@component
PersonaList - Grid display of persona cards with loading and empty states.

Renders personas in a responsive grid layout. Handles loading state with
spinner and empty state with helpful messaging.
-->
<script lang="ts">
	import type { Persona } from '$lib/types';
	import { Spinner } from '$lib/components/ui';
	import PersonaCard from './PersonaCard.svelte';

	/**
	 * @property personas - Array of personas to display
	 * @property isLoading - Shows loading spinner when true
	 * @property emptyMessage - Custom message when no personas exist
	 * @property onSelect - Callback when a persona card is clicked
	 * @property onEdit - Callback for edit action
	 * @property onDelete - Callback for delete action
	 * @property onDuplicate - Callback for duplicate action
	 */
	interface Props {
		personas: Persona[];
		isLoading?: boolean;
		emptyMessage?: string;
		onSelect?: (persona: Persona) => void;
		onEdit?: (persona: Persona) => void;
		onDelete?: (persona: Persona) => void;
		onDuplicate?: (persona: Persona) => void;
	}

	let {
		personas,
		isLoading = false,
		emptyMessage,
		onSelect,
		onEdit,
		onDelete,
		onDuplicate
	}: Props = $props();
</script>

{#if isLoading}
	<div class="flex items-center justify-center py-12">
		<Spinner size="lg" class="text-primary" />
		<span class="ml-3 text-base-content/70">Loading personas...</span>
	</div>
{:else if personas.length === 0}
	<div class="py-12 text-center">
		<svg
			class="mx-auto h-12 w-12 text-base-content/50"
			fill="none"
			viewBox="0 0 24 24"
			stroke="currentColor"
		>
			<path
				stroke-linecap="round"
				stroke-linejoin="round"
				stroke-width="2"
				d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"
			/>
		</svg>
		<h3 class="mt-2 text-sm font-medium text-base-content">
			{emptyMessage ?? 'No personas'}
		</h3>
		{#if !emptyMessage}
			<p class="mt-1 text-sm text-base-content/60">Get started by creating a new persona.</p>
		{/if}
	</div>
{:else}
	<div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
		{#each personas as persona (persona.id)}
			<PersonaCard {persona} {onSelect} {onEdit} {onDelete} {onDuplicate} />
		{/each}
	</div>
{/if}
