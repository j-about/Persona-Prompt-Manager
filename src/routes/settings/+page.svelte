<!--
@component
Settings Page - Application configuration and data management.

Provides interfaces for:
- Configuring AI provider API keys (stored in system keyring)
- Exporting the database as a SQLite file backup
- Importing a database file (replaces all existing data)

@route /settings
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { invalidateAll } from '$app/navigation';
	import { Card, Button, ApiKeyModal } from '$lib/components/ui';
	import { configStore } from '$lib/stores';
	import { exportDatabase, importDatabase } from '$lib/services/export';
	import { getApiKeyStatus, deleteApiKey, type ApiKeyStatus } from '$lib/services/settings';
	import type { AiProvider } from '$lib/types';

	// State
	let isExporting = $state(false);
	let isImporting = $state(false);
	let importError = $state<string | null>(null);
	let importSuccess = $state<string | null>(null);

	// API Key state
	let apiKeyStatuses = $state<ApiKeyStatus[]>([]);
	let showApiKeyModal = $state(false);
	let selectedProvider = $state<AiProvider | null>(null);

	// Helper to check if a provider has a key configured
	function hasKey(provider: AiProvider): boolean {
		const status = apiKeyStatuses.find((s) => s.provider === provider);
		return status?.has_key ?? false;
	}

	async function loadApiKeyStatus() {
		try {
			apiKeyStatuses = await getApiKeyStatus();
		} catch (error) {
			console.error('Failed to load API key status:', error);
		}
	}

	function openApiKeyModal(provider: AiProvider) {
		selectedProvider = provider;
		showApiKeyModal = true;
	}

	function closeApiKeyModal() {
		showApiKeyModal = false;
		selectedProvider = null;
	}

	async function handleDeleteApiKey(provider: AiProvider) {
		const providerInfo = configStore.getProviderById(provider);
		if (!confirm(`Remove API key for ${providerInfo?.displayName ?? provider}?`)) return;

		try {
			await deleteApiKey(provider);
			await loadApiKeyStatus();
		} catch (error) {
			console.error('Failed to delete API key:', error);
		}
	}

	onMount(() => {
		loadApiKeyStatus();
	});

	async function handleExportAll() {
		isExporting = true;
		importError = null;
		importSuccess = null;

		try {
			const result = await exportDatabase();

			if (result.success && result.path) {
				importSuccess = `Database exported to ${result.path}`;
			} else if (result.error) {
				importError = result.error;
			}
			// If neither success nor error, user cancelled - do nothing
		} catch (error) {
			importError = error instanceof Error ? error.message : 'Failed to export database';
		} finally {
			isExporting = false;
		}
	}

	async function handleImport() {
		// Show confirmation dialog
		const confirmed = confirm(
			'WARNING: Importing a database will replace ALL existing data. ' +
				'This action cannot be undone.\n\n' +
				'Do you want to continue?'
		);

		if (!confirmed) return;

		isImporting = true;
		importError = null;
		importSuccess = null;

		try {
			const result = await importDatabase();

			if (result.success) {
				importSuccess = `Successfully imported database with ${result.personas_count} persona(s)`;
				// Invalidate all stores to refresh data
				await invalidateAll();
			} else if (result.error) {
				importError = result.error;
			}
		} catch (error) {
			importError = error instanceof Error ? error.message : 'Failed to import database';
		} finally {
			isImporting = false;
		}
	}
</script>

<div>
	<h1 class="mb-6 text-3xl font-bold text-base-content">Settings</h1>

	<div class="space-y-6">
		<!-- AI Providers -->
		<Card>
			<h2 class="mb-4 text-xl font-semibold text-base-content">AI Providers</h2>
			<p class="mb-4 text-base-content/70">
				Configure API keys for cloud providers. Keys are stored securely in your system keyring.
			</p>

			<div class="space-y-3">
				{#each configStore.aiProviders as provider (provider.id)}
					{@const providerId = provider.id}
					<div
						class="flex items-center justify-between border-b border-base-300 py-3 last:border-0"
					>
						<div class="flex-1">
							<p class="font-medium text-base-content">{provider.displayName}</p>
							<p class="text-sm text-base-content/60">
								{provider.requiresApiKey ? 'Cloud provider' : 'Local provider'}
							</p>
						</div>

						<div class="flex items-center gap-3">
							<!-- Status badge -->
							{#if !provider.requiresApiKey}
								<span class="badge badge-sm badge-neutral"> No key required </span>
							{:else if hasKey(providerId)}
								<span class="badge badge-soft badge-sm badge-success"> Configured </span>
							{:else}
								<span class="badge badge-soft badge-sm badge-warning"> Not configured </span>
							{/if}

							<!-- Action buttons -->
							{#if provider.requiresApiKey}
								{#if hasKey(providerId)}
									<Button variant="ghost" size="sm" onclick={() => openApiKeyModal(providerId)}>
										Edit
									</Button>
									<Button variant="ghost" size="sm" onclick={() => handleDeleteApiKey(providerId)}>
										Remove
									</Button>
								{:else}
									<Button variant="secondary" size="sm" onclick={() => openApiKeyModal(providerId)}>
										Set API Key
									</Button>
								{/if}
							{/if}
						</div>
					</div>
				{/each}
			</div>
		</Card>

		<!-- Import/Export -->
		<Card>
			<h2 class="mb-4 text-xl font-semibold text-base-content">Data Management</h2>
			<p class="mb-4 text-base-content/70">
				Export and import your entire database as a SQLite file for backup or migration.
			</p>

			{#if importError}
				<div role="alert" class="mb-4 alert alert-soft alert-error">
					<span>{importError}</span>
				</div>
			{/if}

			{#if importSuccess}
				<div role="alert" class="mb-4 alert alert-soft alert-success">
					<span>{importSuccess}</span>
				</div>
			{/if}

			<div class="flex gap-4">
				<Button variant="secondary" onclick={handleExportAll} disabled={isExporting}>
					{isExporting ? 'Exporting...' : 'Export Database'}
				</Button>
				<Button variant="secondary" onclick={handleImport} disabled={isImporting}>
					{isImporting ? 'Importing...' : 'Import Database'}
				</Button>
			</div>

			<div class="mt-4 space-y-2">
				<p class="text-sm text-base-content/60">
					Export creates a complete backup of all personas, tokens, and settings.
				</p>
				<p class="text-sm text-warning">Warning: Import will replace ALL existing data.</p>
			</div>
		</Card>
	</div>
</div>

<!-- API Key Modal -->
{#if selectedProvider}
	{@const providerInfo = configStore.getProviderById(selectedProvider)}
	<ApiKeyModal
		bind:open={showApiKeyModal}
		providerId={selectedProvider}
		providerDisplayName={providerInfo?.displayName ?? selectedProvider}
		hasExistingKey={hasKey(selectedProvider)}
		onsave={loadApiKeyStatus}
		onclose={closeApiKeyModal}
	/>
{/if}
