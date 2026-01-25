<script lang="ts">
	import { onMount } from 'svelte';
	import { Card, Button, ApiKeyModal } from '$lib/components/ui';
	import { configStore, personaStore } from '$lib/stores';
	import { getApiKey, getApiKeyStatus, type ApiKeyStatus } from '$lib/services/settings';
	import { generatePersonaWithAi, getAiProviderConfig } from '$lib/services/ai';
	import { getDefaultImageModelId } from '$lib/services/config';
	import { createPersona, updatePersona, updateGenerationParams } from '$lib/services/persona';
	import { createToken } from '$lib/services/token';
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

	// AI improvement preferences (checked by default = improve via AI)
	let improveDescriptionViaAi = $state(true);
	let improveInstructionsViaAi = $state(true);
	// Skip AI description generation when no description provided (unchecked by default = AI generates)
	let skipAiDescription = $state(false);

	// UI state
	let isLoading = $state(false);
	let error = $state<string | null>(null);

	// API key status state
	let apiKeyStatuses = $state<ApiKeyStatus[]>([]);

	// API Key Modal state
	let showApiKeyModal = $state(false);

	// Style suggestions
	const styleSuggestions = [
		'3D render',
		'8K',
		'abstract',
		'acrylic painting',
		'anime',
		'anime style',
		'art deco',
		'art nouveau',
		'baroque',
		'best quality',
		'bokeh',
		'cartoon',
		'cel-shaded',
		'charcoal',
		'chibi',
		'cinematic',
		'colored pencil',
		'concept art',
		'cubism',
		'cyberpunk',
		'dark fantasy',
		'depth of field',
		'detailed',
		'digital art',
		'digital painting',
		'documentary',
		'DSLR',
		'editorial',
		'expressionism',
		'fantasy',
		'fashion photography',
		'film grain',
		'film texture',
		'fresco',
		'golden hour',
		'gothic',
		'gouache',
		'graphite',
		'HDR',
		'high quality',
		'highly detailed',
		'hyper detailed',
		'hyper realistic',
		'illustration',
		'impressionism',
		'ink drawing',
		'ink wash',
		'intricate',
		'long exposure',
		'low poly',
		'macro',
		'manga',
		'masterpiece',
		'matte',
		'minimalist',
		'noir',
		'oil painting',
		'pastel',
		'pen and ink',
		'pencil sketch',
		'photorealistic',
		'pixel art',
		'pop art',
		'portrait photography',
		'professional photography',
		'realistic',
		'renaissance',
		'romanticism',
		'sci-fi',
		'semi-realistic',
		'sharp focus',
		'soft lighting',
		'steampunk',
		'street photography',
		'Studio Ghibli',
		'studio lighting',
		'surrealism',
		'tempera',
		'ultra detailed',
		'vaporwave',
		'vector art',
		'vintage photography',
		'visual novel',
		'volumetric lighting',
		'watercolor'
	];

	// Provider data from config store
	const aiProviders = $derived(
		configStore.aiProviders.map((p) => ({
			id: p.id,
			displayName: p.displayName,
			requiresApiKey: p.requiresApiKey
		}))
	);

	// Image models from config store (pre-sorted by backend)
	const knownModels = $derived(configStore.imageModels);

	// Existing tags from all personas (for AI to prefer over creating new ones)
	const existingTags = $derived([...new Set(personaStore.personas.flatMap((p) => p.tags))].sort());

	// Check if selected provider requires an API key that is missing
	const selectedProviderMissingApiKey = $derived.by(() => {
		if (!aiProviderId) return false;
		const provider = configStore.getProviderById(aiProviderId);
		if (!provider?.requiresApiKey) return false;
		const status = apiKeyStatuses.find((s) => s.provider === aiProviderId);
		return !status?.has_key;
	});

	// Form validation
	const isValid = $derived(
		name.trim().length > 0 &&
			style.trim().length > 0 &&
			aiProviderId !== null &&
			(aiModelId?.trim().length ?? 0) > 0 &&
			!selectedProviderMissingApiKey
	);

	function handleProviderChange() {
		if (aiProviderId) {
			const provider = configStore.getProviderById(aiProviderId);
			aiModelId = provider?.defaultModel ?? null;
		} else {
			aiModelId = null;
		}
	}

	async function handleApiKeySaved() {
		apiKeyStatuses = await getApiKeyStatus();
	}

	// Load default image model, personas, and API key status on mount
	onMount(async () => {
		try {
			const [defaultModel, statuses] = await Promise.all([
				getDefaultImageModelId(),
				getApiKeyStatus()
			]);
			imageModelId = defaultModel;
			apiKeyStatuses = statuses;
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
				characterDescription: characterDescription.trim() || null,
				physicalCriteria: Object.keys(physicalCriteria).length > 0 ? physicalCriteria : undefined,
				aiInstructions: aiInstructions.trim() || undefined,
				imageModelId: imageModelId || undefined,
				existingTags: existingTags.length > 0 ? existingTags : undefined,
				improveDescriptionViaAi: improveDescriptionViaAi,
				improveInstructionsViaAi: improveInstructionsViaAi,
				skipAiDescription: !characterDescription.trim() && skipAiDescription
			};

			// 3. Generate persona with AI
			const aiResponse = await generatePersonaWithAi(config, request);

			// 4. Determine final description: handle skip case, user's original, or AI-improved
			const finalDescription =
				skipAiDescription && !characterDescription.trim()
					? '' // User explicitly chose to skip AI description generation
					: improveDescriptionViaAi
						? aiResponse.description
						: characterDescription.trim();

			// 5. Determine final instructions: use AI-improved if improving, otherwise user's original
			const finalInstructions = improveInstructionsViaAi
				? (aiResponse.aiInstructions ?? (aiInstructions.trim() || null))
				: aiInstructions.trim() || null;

			// 6. Create persona in database
			const persona = await createPersona({
				name: name.trim(),
				description: finalDescription,
				tags: aiResponse.tags
			});

			// 7. Update persona with AI config
			await updatePersona(persona.id, {
				ai_provider_id: aiProviderId,
				ai_model_id: aiModelId,
				ai_instructions: finalInstructions
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
					// Create each token individually to preserve AI-suggested weights
					for (const token of tokens) {
						await createToken({
							persona_id: persona.id,
							granularity_id: granularityId,
							polarity: 'positive',
							content: token.content,
							weight: token.suggested_weight
						});
					}
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
				{#if aiProviderId && selectedProviderMissingApiKey}
					<div role="alert" class="mt-2 alert alert-soft alert-warning">
						<span>
							API key required for {configStore.getProviderById(aiProviderId)?.displayName}.
							<button
								type="button"
								class="link link-primary"
								onclick={() => (showApiKeyModal = true)}
							>
								Configure API Key
							</button>
						</span>
					</div>
				{/if}
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
					{#if aiInstructions.trim()}
						<label class="label mt-2 cursor-pointer justify-start gap-2">
							<input
								type="checkbox"
								class="checkbox checkbox-sm checkbox-primary"
								bind:checked={improveInstructionsViaAi}
							/>
							<span class="label-text">Improve via AI</span>
						</label>
					{/if}
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
				<label for="description" class="label">
					<span class="label-text">Character Description</span>
				</label>
				<textarea
					id="description"
					bind:value={characterDescription}
					rows={5}
					class="textarea-bordered textarea w-full"
					placeholder="Optional: Describe ethnicity, nationality, age, background, biography, personality, and any distinctive physical features. If not provided, AI will derive from style and physical criteria."
				></textarea>
				{#if characterDescription.trim()}
					<label class="label mt-2 cursor-pointer justify-start gap-2">
						<input
							type="checkbox"
							class="checkbox checkbox-sm checkbox-primary"
							bind:checked={improveDescriptionViaAi}
						/>
						<span class="label-text">Improve via AI</span>
					</label>
				{:else}
					<label class="label mt-2 cursor-pointer justify-start gap-2">
						<input
							type="checkbox"
							class="checkbox checkbox-sm checkbox-primary"
							bind:checked={skipAiDescription}
						/>
						<span class="label-text">Skip AI description generation</span>
					</label>
				{/if}
			</div>
		</div>
	</Card>

	<!-- Visual Style -->
	<Card>
		<h2 class="mb-4 text-lg font-semibold text-base-content">Visual Style</h2>
		<p class="mb-4 text-xs text-base-content/60">
			Select or enter the visual art style for image generation.
		</p>
		<div>
			<label for="style" class="label">
				<span class="label-text">Style <span class="text-error">*</span></span>
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

{#if aiProviderId}
	{@const provider = configStore.getProviderById(aiProviderId)}
	<ApiKeyModal
		bind:open={showApiKeyModal}
		providerId={aiProviderId}
		providerDisplayName={provider?.displayName ?? aiProviderId}
		hasExistingKey={false}
		onsave={handleApiKeySaved}
	/>
{/if}
