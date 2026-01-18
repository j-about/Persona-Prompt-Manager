<!--
@component
New Persona Page - Create a persona manually or with AI assistance.

Provides two creation modes via tabs:
- Manual: Basic form for name, description, and tags
- AI-Assisted: Generate a full persona from a description using AI

After creation, redirects to the persona detail page in edit mode
with the created=true flag to show a success toast.

@route /personas/new
-->
<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { PersonaForm, AiPersonaForm } from '$lib/components/persona';
	import { personaStore } from '$lib/stores';
	import type { CreatePersonaRequest, UpdatePersonaRequest } from '$lib/types';

	/** Current active creation mode tab */
	let activeTab = $state<'manual' | 'ai'>('manual');

	/**
	 * Handles manual form submission.
	 * Creates the persona and navigates to its detail page in edit mode.
	 * @param data - The persona creation data from the form
	 */
	async function handleManualSubmit(data: CreatePersonaRequest | UpdatePersonaRequest) {
		const persona = await personaStore.create(data as CreatePersonaRequest);
		if (persona) {
			goto(resolve(`/personas/${persona.id}?edit=true&created=true`));
		}
	}

	/**
	 * Handles successful AI persona creation.
	 * Navigates to the newly created persona's detail page.
	 * @param personaId - The ID of the created persona
	 */
	function handleAiCreated(personaId: string) {
		goto(resolve(`/personas/${personaId}?edit=true&created=true`));
	}

	/** Navigates back to the personas list */
	function handleCancel() {
		goto(resolve('/personas'));
	}
</script>

<div>
	<div class="mb-6">
		<a href={resolve('/personas')} class="text-primary hover:brightness-90">
			&larr; Back to Personas
		</a>
	</div>

	<h1 class="mb-6 text-3xl font-bold text-base-content">Create New Persona</h1>

	{#if personaStore.error}
		<div role="alert" class="mb-4 alert alert-soft alert-error">
			<span>{personaStore.error}</span>
			<button type="button" class="btn btn-ghost btn-xs" onclick={() => personaStore.clearError()}>
				Dismiss
			</button>
		</div>
	{/if}

	<!-- Centered tabs -->
	<div class="mb-6 flex justify-center">
		<div class="tabs-box tabs">
			<input
				type="radio"
				name="creation_mode"
				class="tab"
				aria-label="Manual"
				checked={activeTab === 'manual'}
				onchange={() => (activeTab = 'manual')}
			/>
			<input
				type="radio"
				name="creation_mode"
				class="tab"
				aria-label="AI-Assisted"
				checked={activeTab === 'ai'}
				onchange={() => (activeTab = 'ai')}
			/>
		</div>
	</div>

	<!-- Tab content -->
	{#if activeTab === 'manual'}
		<PersonaForm
			isLoading={personaStore.isLoading}
			onSubmit={handleManualSubmit}
			onCancel={handleCancel}
		/>
	{:else}
		<AiPersonaForm onCreated={handleAiCreated} onCancel={handleCancel} />
	{/if}
</div>
