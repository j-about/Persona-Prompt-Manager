<!--
@component
Persona Detail Page - View and edit a single persona with its tokens.

Displays persona details in read-only mode by default. When editing, uses
the token store's draft mode to stage changes until Save is clicked. This
allows users to add/edit/delete multiple tokens before committing all changes.

Draft Mode Flow:
1. startEdit() - Initializes draft state with current tokens
2. Token changes are staged via draftCreateTokensBatch/draftUpdateToken/draftDeleteToken
3. handleUpdate() - Commits draft to backend, then saves persona
4. handleCancelEdit() - Discards all staged changes

@route /personas/[id]
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { Card, Button } from '$lib/components/ui';
	import { PersonaForm } from '$lib/components/persona';
	import { TokenManager } from '$lib/components/token';
	import { configStore, personaStore, toastStore, tokenStore } from '$lib/stores';
	import { getGenerationParams } from '$lib/services/persona';
	import type {
		UpdatePersonaRequest,
		UpdateTokenRequest,
		TokenPolarity,
		GenerationParams
	} from '$lib/types';

	/** Whether the page is in edit mode */
	let isEditing = $state(false);
	/** Controls visibility of the delete confirmation dialog */
	let showDeleteConfirm = $state(false);
	/** Image generation parameters for this persona */
	let generationParams = $state<GenerationParams | null>(null);

	/** Persona ID extracted from route params */
	const personaId = $derived($page.params.id);

	/**
	 * Loads persona and token data on mount.
	 * Also handles the created=true param to show success toast after creation.
	 */
	onMount(async () => {
		const createdParam = $page.url.searchParams.get('created');
		if (createdParam === 'true') {
			toastStore.success('Persona created! Configure generation settings and add tokens below.');
			const url = new URL($page.url);
			url.searchParams.delete('created');
			goto(resolve((url.pathname + url.search) as `/personas/${string}`), { replaceState: true });
		}

		if (personaId) {
			await Promise.all([
				personaStore.loadById(personaId),
				tokenStore.loadTokensForPersona(personaId)
			]);

			try {
				generationParams = await getGenerationParams(personaId);
			} catch (error) {
				console.error('Failed to load generation params:', error);
			}

			// Handle edit mode from URL parameter (after data is loaded)
			const editParam = $page.url.searchParams.get('edit');
			if (editParam === 'true') {
				tokenStore.startDraft();
				isEditing = true;
			}
		}
	});

	/** Currently loaded persona from the store */
	const persona = $derived(personaStore.selectedPersona);

	/** Formatted creation timestamp for display */
	const formattedCreatedAt = $derived(
		persona
			? new Date(persona.created_at).toLocaleDateString('en-US', {
					year: 'numeric',
					month: 'long',
					day: 'numeric',
					hour: '2-digit',
					minute: '2-digit'
				})
			: ''
	);

	/** Formatted last update timestamp for display */
	const formattedUpdatedAt = $derived(
		persona
			? new Date(persona.updated_at).toLocaleDateString('en-US', {
					year: 'numeric',
					month: 'long',
					day: 'numeric',
					hour: '2-digit',
					minute: '2-digit'
				})
			: ''
	);

	/**
	 * Resolves AI provider ID to human-readable display name.
	 * @param providerId - The provider identifier
	 * @returns Display name or the ID if not found
	 */
	function getProviderDisplayName(providerId: string): string {
		const info = configStore.getProviderById(providerId);
		return info?.displayName ?? providerId;
	}

	/**
	 * Saves persona changes including staged token modifications.
	 * First commits the token draft, then updates the persona.
	 * Aborts if token commit fails.
	 * @param data - Persona fields to update
	 */
	async function handleUpdate(data: UpdatePersonaRequest) {
		if (persona) {
			const tokenSuccess = await tokenStore.commitDraft();
			if (!tokenSuccess) {
				return;
			}

			const updated = await personaStore.update(persona.id, data);
			if (updated) {
				isEditing = false;
				goto(resolve(`/personas/${persona.id}`), { replaceState: true });
				try {
					generationParams = await getGenerationParams(persona.id);
				} catch (error) {
					console.error('Failed to reload generation params:', error);
				}
			}
		}
	}

	/**
	 * Cancels edit mode and discards all staged token changes.
	 * Navigates back to the read-only view.
	 */
	function handleCancelEdit() {
		tokenStore.discardDraft();
		isEditing = false;
		goto(resolve(`/personas/${personaId}`), { replaceState: true });
	}

	/** Deletes the persona and navigates to the list */
	async function handleDelete() {
		if (persona) {
			const success = await personaStore.remove(persona.id);
			if (success) {
				goto(resolve('/personas'));
			}
		}
	}

	/**
	 * Enters edit mode and initializes the token draft state.
	 * Draft mode allows staging multiple token changes before commit.
	 */
	function startEdit() {
		tokenStore.startDraft();
		isEditing = true;
	}

	/**
	 * Stages new tokens in draft mode.
	 * Changes are not persisted until handleUpdate() commits the draft.
	 */
	async function handleCreateToken(data: {
		personaId: string;
		granularityId: string;
		polarity: TokenPolarity;
		contents: string;
		weight: number;
	}): Promise<void> {
		tokenStore.draftCreateTokensBatch({
			persona_id: data.personaId,
			granularity_id: data.granularityId,
			polarity: data.polarity,
			contents: data.contents,
			weight: data.weight
		});
	}

	/**
	 * Stages a token update in draft mode.
	 * @param id - Token ID to update
	 * @param data - Fields to modify
	 */
	async function handleUpdateToken(id: string, data: UpdateTokenRequest): Promise<void> {
		tokenStore.draftUpdateToken(id, data);
	}

	/**
	 * Stages a token deletion in draft mode.
	 * @param id - Token ID to delete
	 */
	async function handleDeleteToken(id: string): Promise<void> {
		tokenStore.draftDeleteToken(id);
	}
</script>

<div>
	<div class="mb-6">
		<a href={resolve('/personas')} class="text-primary hover:brightness-90">
			&larr; Back to Personas
		</a>
	</div>

	{#if personaStore.error}
		<div role="alert" class="mb-4 alert alert-soft alert-error">
			<span>{personaStore.error}</span>
			<button type="button" class="btn btn-ghost btn-xs" onclick={() => personaStore.clearError()}>
				Dismiss
			</button>
		</div>
	{/if}

	{#if personaStore.isLoading && !persona}
		<div class="flex items-center justify-center py-12">
			<span class="loading loading-md loading-spinner text-primary"></span>
			<span class="ml-3 text-base-content/70">Loading persona...</span>
		</div>
	{:else if !persona}
		<Card>
			<p class="text-base-content/70">Persona not found.</p>
			<a href={resolve('/personas')} class="mt-2 inline-block text-primary hover:underline">
				Return to personas list
			</a>
		</Card>
	{:else if isEditing}
		<PersonaForm
			{persona}
			isLoading={personaStore.isLoading}
			onSubmit={handleUpdate}
			onCancel={handleCancelEdit}
		>
			{#snippet tokenSection()}
				<TokenManager
					personaId={persona.id}
					tokens={tokenStore.draftTokens}
					granularityLevels={tokenStore.granularityLevels}
					isLoading={tokenStore.isLoading}
					isReadOnly={false}
					onCreateToken={handleCreateToken}
					onUpdateToken={handleUpdateToken}
					onDeleteToken={handleDeleteToken}
				/>
			{/snippet}
		</PersonaForm>
	{:else}
		<div class="mb-6 flex items-start justify-between">
			<div>
				<h1 class="text-3xl font-bold text-base-content">{persona.name}</h1>
				{#if persona.tags.length > 0}
					<div class="mt-2 flex flex-wrap gap-2">
						{#each persona.tags as tag (tag)}
							<span class="badge badge-soft badge-primary">
								{tag}
							</span>
						{/each}
					</div>
				{/if}
			</div>
			<div class="flex gap-2">
				<Button variant="secondary" onclick={startEdit}>Edit</Button>
				<Button variant="danger" onclick={() => (showDeleteConfirm = true)}>Delete</Button>
			</div>
		</div>

		<div class="space-y-6">
			{#if persona.description}
				<Card>
					<h2 class="mb-2 text-lg font-semibold text-base-content">Description</h2>
					<p class="whitespace-pre-wrap text-base-content/70 select-text">
						{persona.description}
					</p>
				</Card>
			{/if}

			<!-- Generation Parameters Card -->
			{#if generationParams}
				<Card>
					<h2 class="mb-4 text-lg font-semibold text-base-content">Generation Parameters</h2>
					<dl class="grid grid-cols-3 gap-4 text-sm">
						<div>
							<dt class="font-medium text-base-content/60">Model</dt>
							<dd
								class="mt-1 truncate text-base-content select-text"
								title={generationParams.model_id}
							>
								{generationParams.model_id.split('/').pop()}
							</dd>
						</div>
						<div>
							<dt class="font-medium text-base-content/60">Steps</dt>
							<dd class="mt-1 text-base-content select-text">{generationParams.steps}</dd>
						</div>
						<div>
							<dt class="font-medium text-base-content/60">CFG Scale</dt>
							<dd class="mt-1 text-base-content select-text">{generationParams.cfg_scale}</dd>
						</div>
						<div>
							<dt class="font-medium text-base-content/60">Seed</dt>
							<dd class="mt-1 text-base-content select-text">
								{generationParams.seed === -1 ? 'Random' : generationParams.seed}
							</dd>
						</div>
						{#if generationParams.sampler}
							<div>
								<dt class="font-medium text-base-content/60">Sampler</dt>
								<dd class="mt-1 text-base-content select-text">{generationParams.sampler}</dd>
							</div>
						{/if}
						{#if generationParams.scheduler}
							<div>
								<dt class="font-medium text-base-content/60">Scheduler</dt>
								<dd class="mt-1 text-base-content capitalize select-text">
									{generationParams.scheduler}
								</dd>
							</div>
						{/if}
					</dl>
				</Card>
			{/if}

			<TokenManager
				personaId={persona.id}
				tokens={tokenStore.tokens}
				granularityLevels={tokenStore.granularityLevels}
				isLoading={tokenStore.isLoading}
				isReadOnly={true}
			/>

			<!-- AI Token Generation Card (only if configured) -->
			{#if persona.ai_provider_id}
				<Card>
					<h2 class="mb-4 text-lg font-semibold text-base-content">AI Token Generation</h2>
					<dl class="space-y-3 text-sm">
						<div class="flex justify-between">
							<dt class="font-medium text-base-content/60">Provider</dt>
							<dd class="text-base-content">
								{getProviderDisplayName(persona.ai_provider_id)}
							</dd>
						</div>
						{#if persona.ai_model_id}
							<div class="flex justify-between">
								<dt class="font-medium text-base-content/60">Model</dt>
								<dd class="text-base-content">{persona.ai_model_id}</dd>
							</div>
						{/if}
						{#if persona.ai_instructions}
							<div>
								<dt class="mb-1 font-medium text-base-content/60">Custom Instructions</dt>
								<dd
									class="rounded-lg bg-base-200 p-3 text-sm whitespace-pre-wrap text-base-content select-text"
								>
									{persona.ai_instructions}
								</dd>
							</div>
						{/if}
					</dl>
				</Card>
			{/if}

			<Card>
				<h2 class="mb-4 text-lg font-semibold text-base-content">Details</h2>
				<dl class="grid grid-cols-2 gap-4">
					<div>
						<dt class="text-sm font-medium text-base-content/60">Created</dt>
						<dd class="text-sm text-base-content">{formattedCreatedAt}</dd>
					</div>
					<div>
						<dt class="text-sm font-medium text-base-content/60">Last Updated</dt>
						<dd class="text-sm text-base-content">{formattedUpdatedAt}</dd>
					</div>
					<div>
						<dt class="text-sm font-medium text-base-content/60">ID</dt>
						<dd class="font-mono text-sm text-base-content select-text">{persona.id}</dd>
					</div>
				</dl>
			</Card>
		</div>
	{/if}
</div>

{#if showDeleteConfirm && persona}
	<dialog class="modal-open modal">
		<div class="modal-box">
			<h3 class="text-lg font-bold">Delete Persona</h3>
			<p class="py-4">
				Are you sure you want to delete <strong>{persona.name}</strong>? This will also delete all
				associated tokens. This action cannot be undone.
			</p>
			<div class="modal-action">
				<Button variant="secondary" onclick={() => (showDeleteConfirm = false)}>Cancel</Button>
				<Button variant="danger" onclick={handleDelete}>Delete</Button>
			</div>
		</div>
		<form method="dialog" class="modal-backdrop">
			<button onclick={() => (showDeleteConfirm = false)}>close</button>
		</form>
	</dialog>
{/if}
