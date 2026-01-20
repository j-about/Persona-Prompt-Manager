<script lang="ts">
	import { onMount } from 'svelte';
	import { Card, Button, ApiKeyModal } from '$lib/components/ui';
	import { configStore, tokenStore } from '$lib/stores';
	import {
		exportAllPersonas,
		downloadAsJson,
		parseImportJson,
		importPersonas
	} from '$lib/services/export';
	import { getApiKeyStatus, deleteApiKey, type ApiKeyStatus } from '$lib/services/settings';
	import type { AiProvider, ImportOptions } from '$lib/types';

	// State
	let isExporting = $state(false);
	let isImporting = $state(false);
	let importError = $state<string | null>(null);
	let importSuccess = $state<string | null>(null);
	let fileInput: HTMLInputElement;

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
		try {
			const exportData = await exportAllPersonas();
			downloadAsJson(exportData);
		} catch (error) {
			console.error('Export failed:', error);
		} finally {
			isExporting = false;
		}
	}

	function handleImportClick() {
		fileInput?.click();
	}

	async function handleFileSelected(event: Event) {
		const input = event.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;

		isImporting = true;
		importError = null;
		importSuccess = null;

		try {
			const text = await file.text();
			const exportData = await parseImportJson(text);

			const options: ImportOptions = {
				on_conflict: 'rename',
				import_granularities: true
			};

			const results = await importPersonas(exportData, options);

			const successful = results.filter((r) => r.success).length;
			const failed = results.filter((r) => !r.success).length;

			if (failed === 0) {
				importSuccess = `Successfully imported ${successful} persona(s)`;
			} else {
				importSuccess = `Imported ${successful} persona(s), ${failed} failed`;
			}

			// Reload granularity levels in case new ones were added
			await tokenStore.loadGranularityLevels();
		} catch (error) {
			importError = error instanceof Error ? error.message : 'Failed to import file';
		} finally {
			isImporting = false;
			input.value = '';
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
				Import and export your personas and tokens as JSON files.
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
					{isExporting ? 'Exporting...' : 'Export All Data'}
				</Button>
				<Button variant="secondary" onclick={handleImportClick} disabled={isImporting}>
					{isImporting ? 'Importing...' : 'Import Data'}
				</Button>
			</div>

			<input
				type="file"
				accept=".json"
				class="hidden"
				bind:this={fileInput}
				onchange={handleFileSelected}
			/>

			<p class="mt-4 text-sm text-base-content/60">
				Exported files include all personas, their tokens, and generation parameters.
			</p>
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
