<script lang="ts">
	import { onMount } from 'svelte';
	import type { Persona, CreatePersonaRequest, UpdatePersonaRequest } from '$lib/types';
	import type { Snippet } from 'svelte';
	import { configStore } from '$lib/stores';
	import { Card, Button } from '$lib/components/ui';
	import { getGenerationParams, updateGenerationParams } from '$lib/services/persona';
	import { getDefaultImageModelId } from '$lib/services/config';

	interface Props {
		persona?: Persona | null;
		isLoading?: boolean;
		onSubmit: (data: CreatePersonaRequest | UpdatePersonaRequest) => void;
		onCancel: () => void;
		tokenSection?: Snippet;
	}

	let { persona = null, isLoading = false, onSubmit, onCancel, tokenSection }: Props = $props();

	// Form state - Basic info
	let name = $state('');
	let description = $state('');
	let tagsInput = $state('');

	// Form state - AI Token Generation (optional)
	let aiProviderId = $state<string | null>(null);
	let aiModelId = $state<string | null>(null);
	let aiInstructions = $state('');

	// Sync form state when persona prop changes
	$effect(() => {
		name = persona?.name ?? '';
		description = persona?.description ?? '';
		tagsInput = persona?.tags.join(', ') ?? '';
		aiProviderId = persona?.ai_provider_id ?? null;
		aiModelId = persona?.ai_model_id ?? null;
		aiInstructions = persona?.ai_instructions ?? '';
	});

	// Form state - Generation Parameters (model ID loaded from backend)
	let modelId = $state('');
	let seed = $state(-1);
	let steps = $state(30);
	let cfgScale = $state(7.0);
	let sampler = $state<string | null>(null);
	let scheduler = $state<string | null>(null);

	const isEditMode = $derived(persona !== null);
	const submitLabel = $derived(isEditMode ? 'Save Changes' : 'Create Persona');

	// Validation - model is required when provider is selected
	const isValid = $derived(
		name.trim().length > 0 && (!aiProviderId || (aiModelId?.trim().length ?? 0) > 0)
	);

	// Data from config store (pre-sorted by backend)
	const knownModels = $derived(configStore.imageModels);

	const aiProviders = $derived(
		configStore.aiProviders.map((p) => ({
			id: p.id,
			displayName: p.displayName,
			requiresApiKey: p.requiresApiKey
		}))
	);

	// Load generation params on mount
	onMount(async () => {
		if (persona) {
			// Edit mode: load existing generation params
			try {
				const params = await getGenerationParams(persona.id);
				if (params) {
					modelId = params.model_id;
					seed = params.seed;
					steps = params.steps;
					cfgScale = params.cfg_scale;
					sampler = params.sampler;
					scheduler = params.scheduler;
				}
			} catch (error) {
				console.error('Failed to load generation params:', error);
			}
		} else {
			// Create mode: load default model ID from backend
			try {
				modelId = await getDefaultImageModelId();
			} catch (error) {
				console.error('Failed to load default model ID:', error);
			}
		}
	});

	async function handleSubmit(e: Event) {
		e.preventDefault();
		if (!isValid || isLoading) return;

		const tags = tagsInput
			.split(',')
			.map((t) => t.trim())
			.filter((t) => t.length > 0);

		if (isEditMode && persona) {
			const updateData: UpdatePersonaRequest = {};
			if (name !== persona.name) updateData.name = name.trim();
			if (description !== (persona.description ?? ''))
				updateData.description = description.trim() || null;
			if (tagsInput !== persona.tags.join(', ')) updateData.tags = tags;

			// AI fields - when provider is cleared, also clear model and instructions
			if (aiProviderId !== persona.ai_provider_id) {
				updateData.ai_provider_id = aiProviderId;
				if (!aiProviderId) {
					// Provider cleared - ensure model and instructions are also cleared in DB
					updateData.ai_model_id = null;
					updateData.ai_instructions = null;
				}
			}
			if (aiProviderId && aiModelId !== persona.ai_model_id) {
				updateData.ai_model_id = aiModelId;
			}
			if (aiInstructions !== (persona.ai_instructions ?? '')) {
				updateData.ai_instructions = aiInstructions.trim() || null;
			}

			// Update generation params separately
			try {
				await updateGenerationParams({
					persona_id: persona.id,
					model_id: modelId,
					seed,
					steps,
					cfg_scale: cfgScale,
					sampler: sampler || null,
					scheduler: scheduler || null
				});
			} catch (error) {
				console.error('Failed to update generation params:', error);
			}

			onSubmit(updateData);
		} else {
			const createData: CreatePersonaRequest = {
				name: name.trim(),
				description: description.trim() || null,
				tags
			};
			onSubmit(createData);
		}
	}

	// Handle AI provider change - auto-populate model or clear fields
	function handleProviderChange() {
		if (aiProviderId) {
			// Auto-populate with provider's default model
			const provider = configStore.getProviderById(aiProviderId);
			aiModelId = provider?.defaultModel ?? null;
		} else {
			// Clear all AI fields when provider is deselected
			aiModelId = null;
			aiInstructions = '';
		}
	}
</script>

<form onsubmit={handleSubmit} class="space-y-6">
	<!-- Basic Information -->
	<Card>
		<h2 class="mb-4 text-lg font-semibold text-base-content">Basic Information</h2>
		<div class="space-y-4">
			<div>
				<label for="name" class="block text-sm font-medium text-base-content">
					Name <span class="text-error">*</span>
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
				<label for="description" class="block text-sm font-medium text-base-content">
					Description
				</label>
				<textarea
					id="description"
					bind:value={description}
					rows={3}
					class="textarea-bordered textarea w-full"
					placeholder="Enter a brief description of the persona"
				></textarea>
			</div>

			<div>
				<label for="tags" class="block text-sm font-medium text-base-content"> Tags </label>
				<input
					type="text"
					id="tags"
					bind:value={tagsInput}
					class="input-bordered input w-full"
					placeholder="Enter tags separated by commas"
				/>
				<p class="mt-1 text-xs text-base-content/60">
					Separate multiple tags with commas (e.g., fantasy, female, warrior)
				</p>
			</div>
		</div>
	</Card>

	<!-- Edit mode only sections -->
	{#if isEditMode}
		<!-- Image Generation Settings -->
		<Card>
			<h2 class="mb-4 text-lg font-semibold text-base-content">Image Generation Settings</h2>
			<div class="space-y-4">
				<div>
					<label for="modelId" class="block text-sm font-medium text-base-content">
						Image Generation Model <span class="text-error">*</span>
					</label>
					<select id="modelId" bind:value={modelId} required class="select-bordered select w-full">
						{#each knownModels as model (model.model_id)}
							<option value={model.model_id}
								>{model.model_id.split('/').pop()} ({model.max_tokens} tokens)</option
							>
						{/each}
						{#if knownModels.length === 0}
							<option value={modelId}>{modelId}</option>
						{/if}
					</select>
					<p class="mt-1 text-xs text-base-content/60">
						Determines which tokenizer is used for token counting
					</p>
				</div>

				<div class="grid grid-cols-2 gap-4">
					<div>
						<label for="seed" class="block text-sm font-medium text-base-content"> Seed </label>
						<input
							type="number"
							id="seed"
							bind:value={seed}
							class="input-bordered input w-full"
							placeholder="-1 = random"
						/>
					</div>

					<div>
						<label for="steps" class="block text-sm font-medium text-base-content"> Steps </label>
						<input
							type="number"
							id="steps"
							bind:value={steps}
							min={1}
							max={150}
							class="input-bordered input w-full"
						/>
					</div>

					<div>
						<label for="cfgScale" class="block text-sm font-medium text-base-content">
							CFG Scale
						</label>
						<input
							type="number"
							id="cfgScale"
							bind:value={cfgScale}
							step={0.5}
							min={1}
							max={30}
							class="input-bordered input w-full"
						/>
					</div>

					<div>
						<label for="sampler" class="block text-sm font-medium text-base-content">
							Sampler
						</label>
						<input
							type="text"
							id="sampler"
							bind:value={sampler}
							class="input-bordered input w-full"
							placeholder="e.g., DPM++ 2M"
						/>
					</div>

					<div class="col-span-2">
						<label for="scheduler" class="block text-sm font-medium text-base-content">
							Scheduler
						</label>
						<select id="scheduler" bind:value={scheduler} class="select-bordered select w-full">
							<option value={null}>Default</option>
							<option value="normal">Normal</option>
							<option value="karras">Karras</option>
							<option value="exponential">Exponential</option>
							<option value="sgm_uniform">SGM Uniform</option>
						</select>
					</div>
				</div>
			</div>
		</Card>

		<!-- Token Section (slotted content - TokenManager is already a Card) -->
		{#if tokenSection}
			{@render tokenSection()}
		{/if}

		<!-- AI Token Generation (Optional) -->
		<Card>
			<h2 class="mb-4 text-lg font-semibold text-base-content">AI Token Generation (Optional)</h2>
			<p class="mb-4 text-xs text-base-content/60">
				Configure an AI provider to enable automatic token generation for this persona.
			</p>
			<div class="space-y-4">
				<div>
					<label for="aiProviderId" class="block text-sm font-medium text-base-content">
						AI Provider
					</label>
					<select
						id="aiProviderId"
						bind:value={aiProviderId}
						onchange={handleProviderChange}
						class="select-bordered select w-full"
					>
						<option value={null}>Not configured</option>
						{#each aiProviders as provider (provider.id)}
							<option value={provider.id}>{provider.displayName}</option>
						{/each}
					</select>
				</div>

				{#if aiProviderId}
					<div>
						<label for="aiModelId" class="block text-sm font-medium text-base-content">
							AI Model <span class="text-error">*</span>
						</label>
						<input
							type="text"
							id="aiModelId"
							bind:value={aiModelId}
							required
							class="input-bordered input w-full"
							placeholder={configStore.getProviderById(aiProviderId)?.defaultModel ?? ''}
						/>
						<p class="mt-1 text-xs text-base-content/60">
							Enter the model ID to use for token generation
						</p>
					</div>
				{/if}

				<div>
					<label for="aiInstructions" class="block text-sm font-medium text-base-content">
						Custom Instructions
					</label>
					<textarea
						id="aiInstructions"
						bind:value={aiInstructions}
						rows={3}
						class="textarea-bordered textarea w-full"
						placeholder="Optional: Instructions for the AI when generating tokens for this persona..."
					></textarea>
					<p class="mt-1 text-xs text-base-content/60">
						Examples: "This character is from a medieval fantasy setting" or "Focus on subtle,
						natural features"
					</p>
				</div>
			</div>
		</Card>
	{/if}

	<!-- Form Actions -->
	<div class="flex justify-end gap-3">
		<Button type="button" variant="secondary" onclick={onCancel} disabled={isLoading}>
			Cancel
		</Button>
		<Button type="submit" disabled={!isValid || isLoading}>
			{#if isLoading}
				<svg class="mr-2 -ml-1 h-4 w-4 animate-spin" fill="none" viewBox="0 0 24 24">
					<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"
					></circle>
					<path
						class="opacity-75"
						fill="currentColor"
						d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
					></path>
				</svg>
				Saving...
			{:else}
				{submitLabel}
			{/if}
		</Button>
	</div>
</form>
