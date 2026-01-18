<script lang="ts">
	import type { Snippet } from 'svelte';
	import Modal from './Modal.svelte';
	import Button from './Button.svelte';

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
