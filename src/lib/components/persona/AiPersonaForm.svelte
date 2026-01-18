<script lang="ts">
	import { onMount } from 'svelte';
	import { Card, Button } from '$lib/components/ui';
	import { configStore, personaStore } from '$lib/stores';
	import { getApiKey } from '$lib/services/settings';
	import { generatePersonaWithAi, getAiProviderConfig } from '$lib/services/ai';
	import { getDefaultImageModelId } from '$lib/services/config';
	import { createPersona, updatePersona, updateGenerationParams } from '$lib/services/persona';
	import { createTokensBatch } from '$lib/services/token';
	import PhysicalCriteriaForm from './PhysicalCriteriaForm.svelte';
	import type {
		AiPersonaGenerationRequest,
		AiProviderConfig,
		PhysicalCriteria,
		GeneratedTokensByGranularity
	} from '$lib/types';

	interface Props {
		onCreated: (personaId: string) => void;
		onCancel: () => void;
	}

	let { onCreated, onCancel }: Props = $props();

	// Form state
	let name = $state('');
	let style = $state('');
	let characterDescription = $state('');
	let aiProviderId = $state<string | null>(null);
	let aiModelId = $state<string | null>(null);
	let aiInstructions = $state('');
	let physicalCriteria = $state<PhysicalCriteria>({});
	let imageModelId = $state('');

	// UI state
	let isLoading = $state(false);
	let error = $state<string | null>(null);

	// Style suggestions
	const styleSuggestions = [
		'realistic',
		'photorealistic',
		'anime',
		'manga',
		'cartoon',
		'oil painting',
		'watercolor',
		'digital art',
		'3D render',
		'concept art',
		'illustration',
		'semi-realistic'
	];

	// Provider data from config store
	const aiProviders = $derived(
		configStore.aiProviders.map((p) => ({
			id: p.id,
			displayName: p.displayName,
			requiresApiKey: p.requiresApiKey
		}))
	);

	// Image models from config store (sorted for UI)
	const knownModels = $derived(
		[...configStore.imageModels].sort((a, b) => a.model_id.localeCompare(b.model_id))
	);

	// Existing tags from all personas (for AI to prefer over creating new ones)
	const existingTags = $derived([...new Set(personaStore.personas.flatMap((p) => p.tags))].sort());

	// Form validation
	const isValid = $derived(
		name.trim().length > 0 &&
			style.trim().length > 0 &&
			characterDescription.trim().length > 0 &&
			aiProviderId !== null &&
			(aiModelId?.trim().length ?? 0) > 0
	);

	function handleProviderChange() {
		if (aiProviderId) {
			const provider = configStore.getProviderById(aiProviderId);
			aiModelId = provider?.defaultModel ?? null;
		} else {
			aiModelId = null;
		}
	}

	// Load default image model and personas on mount
	onMount(async () => {
		try {
			imageModelId = await getDefaultImageModelId();
			// Load personas to get existing tags (if not already loaded)
			if (personaStore.personas.length === 0) {
				await personaStore.loadAll();
			}
		} catch (err) {
			console.error('Failed to initialize:', err);
		}
	});

	async function handleSubmit(e: Event) {
		e.preventDefault();
		if (!isValid || isLoading || !aiProviderId || !aiModelId) return;

		isLoading = true;
		error = null;

		try {
			// 1. Get provider config and API key
			const providerConfig = await getAiProviderConfig(aiProviderId);
			const apiKey = await getApiKey(aiProviderId);

			const config: AiProviderConfig = {
				...providerConfig,
				model: aiModelId,
				api_key: apiKey
			};

			// 2. Build the request
			const request: AiPersonaGenerationRequest = {
				name: name.trim(),
				style: style.trim(),
				characterDescription: characterDescription.trim(),
				physicalCriteria: Object.keys(physicalCriteria).length > 0 ? physicalCriteria : undefined,
				aiInstructions: aiInstructions.trim() || undefined,
				imageModelId: imageModelId || undefined,
				existingTags: existingTags.length > 0 ? existingTags : undefined
			};

			// 3. Generate persona with AI
			const aiResponse = await generatePersonaWithAi(config, request);

			// 4. Create persona in database
			const persona = await createPersona({
				name: name.trim(),
				description: aiResponse.description,
				tags: aiResponse.tags
			});

			// 5. Update persona with AI config (use user-provided instructions)
			await updatePersona(persona.id, {
				ai_provider_id: aiProviderId,
				ai_model_id: aiModelId,
				ai_instructions: aiInstructions.trim() || null
			});

			// 5.5. Set generation parameters with selected image model
			await updateGenerationParams({
				persona_id: persona.id,
				model_id: imageModelId,
				seed: -1,
				steps: 30,
				cfg_scale: 7.0,
				sampler: null,
				scheduler: null
			});

			// 6. Create tokens for each granularity
			const granularityMap: Record<keyof GeneratedTokensByGranularity, string> = {
				style: 'style',
				general: 'general',
				hair: 'hair',
				face: 'face',
				upper_body: 'upper_body',
				midsection: 'midsection',
				lower_body: 'lower_body'
			};

			for (const [key, granularityId] of Object.entries(granularityMap)) {
				const tokens = aiResponse.tokens[key as keyof GeneratedTokensByGranularity];
				if (tokens && tokens.length > 0) {
					// Join token contents with comma for batch creation
					const contents = tokens.map((t) => t.content).join(', ');
					await createTokensBatch({
						persona_id: persona.id,
						granularity_id: granularityId,
						polarity: 'positive',
						contents: contents
					});
				}
			}

			// 7. Success - navigate to the new persona
			onCreated(persona.id);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to generate persona';
		} finally {
			isLoading = false;
		}
	}
</script>

<form onsubmit={handleSubmit} class="space-y-6">
	{#if error}
		<div role="alert" class="alert alert-soft alert-error">
			<span>{error}</span>
			<button type="button" class="btn btn-ghost btn-xs" onclick={() => (error = null)}>
				Dismiss
			</button>
		</div>
	{/if}

	<!-- AI Provider Selection -->
	<Card>
		<h2 class="mb-4 text-lg font-semibold text-base-content">AI Provider</h2>
		<p class="mb-4 text-xs text-base-content/60">
			Select the AI provider and model to use for generating persona details.
		</p>
		<div class="space-y-4">
			<div>
				<label for="aiProvider" class="label">
					<span class="label-text">Provider <span class="text-error">*</span></span>
				</label>
				<select
					id="aiProvider"
					bind:value={aiProviderId}
					onchange={handleProviderChange}
					class="select-bordered select w-full"
					required
				>
					<option value={null}>Select a provider...</option>
					{#each aiProviders as provider (provider.id)}
						<option value={provider.id}>{provider.displayName}</option>
					{/each}
				</select>
			</div>

			{#if aiProviderId}
				<div>
					<label for="aiModel" class="label">
						<span class="label-text">Model <span class="text-error">*</span></span>
					</label>
					<input
						type="text"
						id="aiModel"
						bind:value={aiModelId}
						required
						class="input-bordered input w-full"
						placeholder={configStore.getProviderById(aiProviderId)?.defaultModel ??
							'Enter model ID'}
					/>
					<p class="mt-1 text-xs text-base-content/50">
						Default: {configStore.getProviderById(aiProviderId)?.defaultModel ?? 'N/A'}
					</p>
				</div>

				<div>
					<label for="aiInstructions" class="label">
						<span class="label-text">Custom Instructions</span>
					</label>
					<textarea
						id="aiInstructions"
						bind:value={aiInstructions}
						rows={3}
						class="textarea-bordered textarea w-full"
						placeholder="Optional: Instructions for the AI when generating persona details and tokens..."
					></textarea>
					<p class="mt-1 text-xs text-base-content/50">
						Examples: "Focus on fantasy elements" or "Emphasize ethereal qualities"
					</p>
				</div>
			{/if}
		</div>
	</Card>

	<!-- Image Generation Model -->
	<Card>
		<h2 class="mb-4 text-lg font-semibold text-base-content">Image Generation Model</h2>
		<p class="mb-4 text-xs text-base-content/60">
			Select the model used for image generation. This determines which tokenizer is used for token
			counting.
		</p>
		<div>
			<label for="imageModelId" class="label">
				<span class="label-text">Model <span class="text-error">*</span></span>
			</label>
			<select
				id="imageModelId"
				bind:value={imageModelId}
				required
				class="select-bordered select w-full"
			>
				{#each knownModels as model (model.model_id)}
					<option value={model.model_id}>
						{model.model_id.split('/').pop()} ({model.max_tokens} tokens)
					</option>
				{/each}
				{#if knownModels.length === 0}
					<option value={imageModelId}>{imageModelId}</option>
				{/if}
			</select>
		</div>
	</Card>

	<!-- Character Information -->
	<Card>
		<h2 class="mb-4 text-lg font-semibold text-base-content">Character Information</h2>
		<div class="space-y-4">
			<div>
				<label for="name" class="label">
					<span class="label-text">Name <span class="text-error">*</span></span>
				</label>
				<input
					type="text"
					id="name"
					bind:value={name}
					required
					class="input-bordered input w-full"
					placeholder="Enter persona name"
				/>
			</div>

			<div>
				<label for="style" class="label">
					<span class="label-text">Visual Style <span class="text-error">*</span></span>
				</label>
				<input
					type="text"
					id="style"
					bind:value={style}
					required
					class="input-bordered input w-full"
					placeholder="e.g., realistic, anime, photorealistic, oil painting"
					list="style-suggestions"
				/>
				<datalist id="style-suggestions">
					{#each styleSuggestions as suggestion (suggestion)}
						<option value={suggestion}></option>
					{/each}
				</datalist>
			</div>

			<div>
				<label for="description" class="label">
					<span class="label-text">Character Description <span class="text-error">*</span></span>
				</label>
				<textarea
					id="description"
					bind:value={characterDescription}
					required
					rows={5}
					class="textarea-bordered textarea w-full"
					placeholder="Describe ethnicity, nationality, age, background, biography, personality, and any distinctive physical features. The more detailed, the less you need to specify below."
				></textarea>
			</div>
		</div>
	</Card>

	<!-- Physical Criteria (Optional) -->
	<PhysicalCriteriaForm bind:criteria={physicalCriteria} />

	<!-- Form Actions -->
	<div class="flex justify-end gap-3">
		<Button type="button" variant="secondary" onclick={onCancel} disabled={isLoading}>
			Cancel
		</Button>
		<Button type="submit" disabled={!isValid || isLoading}>
			{#if isLoading}
				<span class="loading mr-2 loading-sm loading-spinner"></span>
				Generating...
			{:else}
				Generate Persona
			{/if}
		</Button>
	</div>
</form>
