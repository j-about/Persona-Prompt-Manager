<!--
@component
Modal - Overlay dialog component.

Renders content in a centered modal with a backdrop.
Clicking the backdrop triggers the onclose callback.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	/**
	 * @property open - Controls modal visibility
	 * @property onclose - Callback when backdrop is clicked
	 * @property class - Additional CSS classes for the modal box
	 * @property children - Modal content
	 */
	interface Props {
		open: boolean;
		onclose?: () => void;
		class?: string;
		children: Snippet;
	}

	let { open, onclose, class: className = '', children }: Props = $props();
</script>

{#if open}
	<dialog class="modal-open modal">
		<div class="modal-box {className}">
			{@render children()}
		</div>
		<form method="dialog" class="modal-backdrop">
			<button type="button" onclick={onclose}>close</button>
		</form>
	</dialog>
{/if}
