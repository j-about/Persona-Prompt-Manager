<!--
@component
Card - Container component with consistent styling.

Provides a shadowed card wrapper using daisyUI's card classes.
Use for grouping related content in a visually distinct container.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	/**
	 * @property class - Additional CSS classes to apply
	 * @property children - Content to render inside the card
	 */
	interface Props {
		class?: string;
		children: Snippet;
	}

	let { class: className = '', children }: Props = $props();
</script>

<div class="card bg-base-100 shadow-md {className}">
	<div class="card-body">
		{@render children()}
	</div>
</div>
