<!--
@component
Tag - Small inline label for displaying persona tags.

Renders a compact, pill-shaped label with neutral styling.
Used primarily in PersonaCard for displaying category tags.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	/**
	 * @property children - Tag label text
	 * @property class - Additional CSS classes
	 */
	interface Props {
		children: Snippet;
		class?: string;
	}

	let { children, class: className = '' }: Props = $props();
</script>

<span
	class="inline-flex items-center bg-base-300 px-2 py-0.5 text-xs font-medium text-base-content {className}"
>
	{@render children()}
</span>
