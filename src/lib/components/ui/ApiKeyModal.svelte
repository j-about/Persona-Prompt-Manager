<!--
@component
ApiKeyModal - Modal for configuring API keys inline.

Allows users to set or update API keys for AI providers without
navigating away from their current form, preserving form data.
-->
<script lang="ts">
	import { Modal, Button } from '$lib/components/ui';
	import { storeApiKey } from '$lib/services/settings';
	import type { AiProvider } from '$lib/types';

	/**
	 * @property open - Controls modal visibility (bindable)
	 * @property providerId - The AI provider ID to configure
	 * @property providerDisplayName - Provider display name for UI text
	 * @property hasExistingKey - Whether the provider already has a key (affects title)
	 * @property onsave - Callback invoked after successful save
	 * @property onclose - Callback invoked when modal is closed
	 */
	interface Props {
		open: boolean;
		providerId: AiProvider;
		providerDisplayName: string;
		hasExistingKey?: boolean;
		onsave?: () => void;
		onclose?: () => void;
	}

	let {
		open = $bindable(),
		providerId,
		providerDisplayName,
		hasExistingKey = false,
		onsave,
		onclose
	}: Props = $props();

	// Internal state
	let apiKeyInput = $state('');
	let isSaving = $state(false);
	let error = $state<string | null>(null);

	// Reset state when modal opens
	$effect(() => {
		if (open) {
			apiKeyInput = '';
			error = null;
		}
	});

	function handleClose() {
		open = false;
		onclose?.();
	}

	async function handleSave() {
		if (!apiKeyInput.trim()) return;

		isSaving = true;
		error = null;

		try {
			await storeApiKey(providerId, apiKeyInput.trim());
			open = false;
			onsave?.();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to save API key';
		} finally {
			isSaving = false;
		}
	}
</script>

<Modal {open} onclose={handleClose}>
	<h3 class="text-lg font-bold">
		{hasExistingKey ? 'Update' : 'Set'} API Key
	</h3>
	<p class="py-2 text-sm text-base-content/70">
		Enter your API key for {providerDisplayName}. The key will be stored securely in your system
		keyring.
	</p>

	{#if error}
		<div role="alert" class="my-4 alert alert-soft alert-error">
			<span>{error}</span>
		</div>
	{/if}

	<div class="form-control mt-4">
		<label for="apiKeyModalInput" class="label">
			<span class="label-text">API Key</span>
		</label>
		<input
			type="password"
			id="apiKeyModalInput"
			bind:value={apiKeyInput}
			placeholder="Enter your API key"
			class="input-bordered input w-full"
			onkeydown={(e) => e.key === 'Enter' && handleSave()}
		/>
	</div>

	<div class="modal-action">
		<Button variant="secondary" onclick={handleClose} disabled={isSaving}>Cancel</Button>
		<Button onclick={handleSave} disabled={isSaving || !apiKeyInput.trim()}>
			{isSaving ? 'Saving...' : 'Save'}
		</Button>
	</div>
</Modal>
