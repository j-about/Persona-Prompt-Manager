<!--
@component
ConfirmDialog - Modal dialog for confirming destructive or important actions.

Displays a title, message content, and action buttons (Cancel/Confirm).
Typically used for delete confirmations and irreversible operations.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import Modal from './Modal.svelte';
	import Button from './Button.svelte';

	/**
	 * @property open - Controls dialog visibility
	 * @property title - Dialog heading text
	 * @property confirmText - Label for confirm button (default: "Confirm")
	 * @property confirmVariant - Style variant for confirm button (default: "danger")
	 * @property onconfirm - Callback when confirm button is clicked
	 * @property oncancel - Callback when cancel button or backdrop is clicked
	 * @property children - Message content to display
	 */
	interface Props {
		open: boolean;
		title: string;
		confirmText?: string;
		confirmVariant?: 'primary' | 'secondary' | 'danger' | 'ghost';
		onconfirm: () => void;
		oncancel: () => void;
		children: Snippet;
	}

	let {
		open,
		title,
		confirmText = 'Confirm',
		confirmVariant = 'danger',
		onconfirm,
		oncancel,
		children
	}: Props = $props();
</script>

<Modal {open} onclose={oncancel}>
	<h3 class="text-lg font-bold">{title}</h3>
	<div class="py-4">
		{@render children()}
	</div>
	<div class="modal-action">
		<Button variant="ghost" onclick={oncancel}>Cancel</Button>
		<Button variant={confirmVariant} onclick={onconfirm}>{confirmText}</Button>
	</div>
</Modal>
