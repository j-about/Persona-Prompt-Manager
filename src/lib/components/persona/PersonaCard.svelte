<!--
@component
PersonaCard - Displays a persona summary with action buttons.

Shows persona name, description (truncated), tags (up to 3), and last update date.
The entire card is clickable for selection, with separate action buttons for
edit, duplicate, and delete operations.
-->
<script lang="ts">
	import type { Persona } from '$lib/types';
	import { Card, Button, Tag } from '$lib/components/ui';

	/**
	 * @property persona - The persona data to display
	 * @property onSelect - Callback when card body is clicked
	 * @property onEdit - Callback for edit button
	 * @property onDelete - Callback for delete button
	 * @property onDuplicate - Callback for duplicate button
	 */
	interface Props {
		persona: Persona;
		onSelect?: (persona: Persona) => void;
		onEdit?: (persona: Persona) => void;
		onDelete?: (persona: Persona) => void;
		onDuplicate?: (persona: Persona) => void;
	}

	let { persona, onSelect, onEdit, onDelete, onDuplicate }: Props = $props();

	/** Formatted last update date for display */
	const formattedDate = $derived(
		new Date(persona.updated_at).toLocaleDateString('en-US', {
			year: 'numeric',
			month: 'short',
			day: 'numeric'
		})
	);

	/** Triggers selection callback when card body is clicked */
	function handleClick() {
		onSelect?.(persona);
	}

	/**
	 * Triggers edit callback.
	 * Stops propagation to prevent card selection.
	 */
	function handleEdit(e: Event) {
		e.stopPropagation();
		onEdit?.(persona);
	}

	/**
	 * Triggers delete callback.
	 * Stops propagation to prevent card selection.
	 */
	function handleDelete(e: Event) {
		e.stopPropagation();
		onDelete?.(persona);
	}

	/**
	 * Triggers duplicate callback.
	 * Stops propagation to prevent card selection.
	 */
	function handleDuplicate(e: Event) {
		e.stopPropagation();
		onDuplicate?.(persona);
	}
</script>

<Card class="cursor-pointer transition-shadow hover:shadow-lg">
	<button type="button" class="w-full text-left" onclick={handleClick}>
		<div class="flex items-start justify-between">
			<div class="min-w-0 flex-1">
				<h3 class="truncate text-lg font-semibold text-base-content">
					{persona.name}
				</h3>
				{#if persona.description}
					<p class="mt-1 line-clamp-2 text-sm text-base-content/70 select-text">
						{persona.description}
					</p>
				{/if}
				{#if persona.tags.length > 0}
					<div class="mt-2 flex flex-wrap gap-1">
						{#each persona.tags.slice(0, 3) as tag (tag)}
							<Tag>{tag}</Tag>
						{/each}
						{#if persona.tags.length > 3}
							<Tag class="text-base-content/70">+{persona.tags.length - 3}</Tag>
						{/if}
					</div>
				{/if}
				<p class="mt-2 text-xs text-base-content/60">
					Updated {formattedDate}
				</p>
			</div>
		</div>
	</button>

	<div class="mt-4 flex gap-2 border-t border-base-300 pt-4">
		<Button variant="secondary" size="sm" onclick={handleEdit}>Edit</Button>
		<Button variant="ghost" size="sm" onclick={handleDuplicate}>Duplicate</Button>
		<Button variant="ghost" size="sm" onclick={handleDelete} class="text-error hover:text-error">
			Delete
		</Button>
	</div>
</Card>
