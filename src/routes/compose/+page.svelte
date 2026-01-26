<!--
@component
Compose Prompt Page - Assembles final prompts from persona tokens.

Allows selecting a persona, filtering by granularity levels, adding adhoc tokens,
and optionally generating AI suggestions. Displays both base prompt (tokens only)
and final prompt (with adhoc additions). Includes token counting for CLIP limits.

@route /compose
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { SvelteSet } from 'svelte/reactivity';
	import { resolve } from '$app/paths';
	import { Card, Button, TokenCountBadge, ApiKeyModal } from '$lib/components/ui';
	import { configStore, personaStore, tokenStore } from '$lib/stores';
	import { composePrompt, copyToClipboard } from '$lib/services/prompt';
	import { countTokens } from '$lib/services/tokenizer';
	import { generateTokens, getAiProviderConfig } from '$lib/services/ai';
	import { getApiKey, getApiKeyStatus, type ApiKeyStatus } from '$lib/services/settings';
	import { getGenerationParams } from '$lib/services/persona';
	import type {
		ComposedPrompt,
		CompositionOptions,
		TokenCount,
		GeneratedToken,
		GenerationParams,
		TokenGenerationRequest
	} from '$lib/types';

	// ==================== Core Composition State ====================
	/** Currently selected persona ID */
	let selectedPersonaId = $state<string | null>(null);
	/** Set of enabled granularity level IDs */
	let selectedGranularityIds = new SvelteSet<string>();
	/** Whether to include weight modifiers in output */
	let includeWeights = $state(true);
	/** User-entered adhoc positive tokens */
	let adhocPositive = $state('');
	/** User-entered adhoc negative tokens */
	let adhocNegative = $state('');
	/** Final composed prompt (base + adhoc) */
	let composedPrompt = $state<ComposedPrompt | null>(null);
	/** Base prompt without adhoc tokens (for display comparison) */
	let basePrompt = $state<ComposedPrompt | null>(null);
	/** Token count for positive prompt (CLIP tokenizer) */
	let positiveTokenCount = $state<TokenCount | null>(null);
	/** Token count for negative prompt (CLIP tokenizer) */
	let negativeTokenCount = $state<TokenCount | null>(null);
	/** Prompt composition in progress */
	let isComposing = $state(false);
	/** Token counting in progress */
	let isCountingTokens = $state(false);
	/** Tracks which copy button shows success state */
	let copySuccess = $state<'positive' | 'negative' | 'both' | null>(null);

	// ==================== AI Token Generation State ====================
	/** User description for AI token context */
	let aiContextDescription = $state('');
	/** AI generation request in progress */
	let isGeneratingAiTokens = $state(false);
	/** Last AI generation error message */
	let aiGenerationError = $state<string | null>(null);
	/** AI-suggested positive tokens awaiting acceptance */
	let aiGeneratedPositive = $state<GeneratedToken[]>([]);
	/** AI-suggested negative tokens awaiting acceptance */
	let aiGeneratedNegative = $state<GeneratedToken[]>([]);

	/** Generation params for the selected persona (provides image model context) */
	let generationParams = $state<GenerationParams | null>(null);

	/** API key status for all providers */
	let apiKeyStatuses = $state<ApiKeyStatus[]>([]);

	/** API Key Modal state */
	let showApiKeyModal = $state(false);

	// ==================== Derived State ====================
	/** Currently selected persona object */
	const selectedPersona = $derived(
		personaStore.personas.find((p) => p.id === selectedPersonaId) ?? null
	);

	/** True when all granularity levels are selected */
	const allGranularitiesSelected = $derived(
		tokenStore.granularityLevels.length > 0 &&
			selectedGranularityIds.size === tokenStore.granularityLevels.length
	);

	/** Personas sorted alphabetically for dropdown display */
	const alphabeticallySortedPersonas = $derived(
		[...personaStore.personas].sort((a, b) => a.name.localeCompare(b.name))
	);

	/** True if persona has both AI provider and model configured */
	const hasAiConfigured = $derived(
		selectedPersona?.ai_provider_id != null && selectedPersona?.ai_model_id != null
	);

	/** Check if the selected persona's AI provider requires an API key that is missing */
	const selectedProviderMissingApiKey = $derived.by(() => {
		if (!selectedPersona?.ai_provider_id) return false;
		const provider = configStore.getProviderById(selectedPersona.ai_provider_id);
		if (!provider?.requiresApiKey) return false;
		const status = apiKeyStatuses.find((s) => s.provider === selectedPersona.ai_provider_id);
		return !status?.has_key;
	});

	/**
	 * Initializes page data: loads personas, granularity levels, API key status,
	 * selects all granularities, and auto-selects first persona.
	 */
	onMount(async () => {
		const [, , statuses] = await Promise.all([
			personaStore.loadAll(),
			tokenStore.loadGranularityLevels(),
			getApiKeyStatus()
		]);
		apiKeyStatuses = statuses;

		for (const level of tokenStore.granularityLevels) {
			selectedGranularityIds.add(level.id);
		}

		if (personaStore.personas.length > 0 && !selectedPersonaId) {
			const sortedPersonas = [...personaStore.personas].sort((a, b) =>
				a.name.localeCompare(b.name)
			);
			if (sortedPersonas[0]) {
				handlePersonaSelect(sortedPersonas[0].id);
			}
		}
	});

	/**
	 * Reactive effect that recomposes the prompt when inputs change.
	 * Explicitly accesses adhoc values to register reactive dependencies.
	 */
	$effect(() => {
		void [adhocPositive, adhocNegative, includeWeights];

		if (selectedPersonaId && selectedGranularityIds.size > 0) {
			composeCurrentPrompt();
		} else {
			composedPrompt = null;
			basePrompt = null;
			positiveTokenCount = null;
			negativeTokenCount = null;
		}
	});

	/**
	 * Composes both base and final prompts via backend service.
	 * Base prompt excludes adhoc tokens; final prompt includes them.
	 * Triggers token counting after composition.
	 */
	async function composeCurrentPrompt() {
		if (!selectedPersonaId) return;

		isComposing = true;
		try {
			const baseOptions: CompositionOptions = {
				include_weights: includeWeights,
				granularity_ids: Array.from(selectedGranularityIds),
				adhoc_positive: null,
				adhoc_negative: null
			};
			basePrompt = await composePrompt(selectedPersonaId, baseOptions);

			const finalOptions: CompositionOptions = {
				include_weights: includeWeights,
				granularity_ids: Array.from(selectedGranularityIds),
				adhoc_positive: adhocPositive.trim() || null,
				adhoc_negative: adhocNegative.trim() || null,
				adhoc_position: 'end'
			};
			composedPrompt = await composePrompt(selectedPersonaId, finalOptions);

			await countPromptTokens();
		} catch (error) {
			console.error('Failed to compose prompt:', error);
		} finally {
			isComposing = false;
		}
	}

	/**
	 * Counts CLIP tokens for both positive and negative prompts.
	 * Used to show token usage relative to model limits.
	 */
	async function countPromptTokens() {
		if (!composedPrompt) return;

		isCountingTokens = true;
		try {
			const [positive, negative] = await Promise.all([
				countTokens(composedPrompt.positive_prompt),
				countTokens(composedPrompt.negative_prompt)
			]);
			positiveTokenCount = positive;
			negativeTokenCount = negative;
		} catch (error) {
			console.error('Failed to count tokens:', error);
		} finally {
			isCountingTokens = false;
		}
	}

	/**
	 * Handles persona selection from dropdown.
	 * Loads the persona's tokens and generation params.
	 * @param personaId - The selected persona ID
	 */
	async function handlePersonaSelect(personaId: string) {
		selectedPersonaId = personaId;
		tokenStore.loadTokensForPersona(personaId);

		try {
			generationParams = await getGenerationParams(personaId);
		} catch (error) {
			console.error('Failed to load generation params:', error);
			generationParams = null;
		}
	}

	/**
	 * Toggles a granularity level in/out of the selection.
	 * @param granularityId - The granularity level to toggle
	 */
	function toggleGranularity(granularityId: string) {
		if (selectedGranularityIds.has(granularityId)) {
			selectedGranularityIds.delete(granularityId);
		} else {
			selectedGranularityIds.add(granularityId);
		}
	}

	/** Toggles between all granularities selected and none selected */
	function toggleAllGranularities() {
		if (allGranularitiesSelected) {
			selectedGranularityIds.clear();
		} else {
			for (const level of tokenStore.granularityLevels) {
				selectedGranularityIds.add(level.id);
			}
		}
	}

	/** Copies positive prompt to clipboard and shows success feedback */
	async function handleCopyPositive() {
		if (composedPrompt?.positive_prompt) {
			await copyToClipboard(composedPrompt.positive_prompt);
			copySuccess = 'positive';
			setTimeout(() => {
				copySuccess = null;
			}, 2000);
		}
	}

	/** Copies negative prompt to clipboard and shows success feedback */
	async function handleCopyNegative() {
		if (composedPrompt?.negative_prompt) {
			await copyToClipboard(composedPrompt.negative_prompt);
			copySuccess = 'negative';
			setTimeout(() => {
				copySuccess = null;
			}, 2000);
		}
	}

	/** Copies both prompts formatted together to clipboard */
	async function handleCopyBoth() {
		if (composedPrompt) {
			const text = `Positive:\n${composedPrompt.positive_prompt}\n\nNegative:\n${composedPrompt.negative_prompt}`;
			await copyToClipboard(text);
			copySuccess = 'both';
			setTimeout(() => {
				copySuccess = null;
			}, 2000);
		}
	}

	/**
	 * Requests AI-generated token suggestions based on context description.
	 * Uses the persona's configured AI provider and includes current prompt
	 * context to avoid duplicates and maintain coherence.
	 */
	async function handleGenerateAiTokens() {
		if (!selectedPersona || !hasAiConfigured || !aiContextDescription.trim()) return;

		isGeneratingAiTokens = true;
		aiGenerationError = null;

		try {
			const config = await getAiProviderConfig(selectedPersona.ai_provider_id!);
			config.model = selectedPersona.ai_model_id!;

			const apiKey = await getApiKey(selectedPersona.ai_provider_id!);
			if (apiKey) {
				config.api_key = apiKey;
			}

			const request: TokenGenerationRequest = {
				persona_name: selectedPersona.name,
				persona_description: selectedPersona.description,
				granularity_name: 'adhoc',
				positive_count: 5,
				negative_count: 3,
				existing_positive_tokens: composedPrompt?.positive_prompt.split(', ').filter(Boolean) ?? [],
				existing_negative_tokens: composedPrompt?.negative_prompt.split(', ').filter(Boolean) ?? [],
				style_hints: aiContextDescription.trim(),
				image_model_id: generationParams?.model_id ?? null,
				ai_instructions: selectedPersona.ai_instructions,
				current_positive_prompt: composedPrompt?.positive_prompt ?? null,
				current_negative_prompt: composedPrompt?.negative_prompt ?? null,
				positive_token_count: positiveTokenCount?.count ?? null,
				negative_token_count: negativeTokenCount?.count ?? null,
				max_usable_tokens: positiveTokenCount?.usable_tokens ?? null
			};

			const response = await generateTokens(config, request);

			aiGeneratedPositive = response.positive_tokens;
			aiGeneratedNegative = response.negative_tokens;
		} catch (error) {
			aiGenerationError = error instanceof Error ? error.message : 'Failed to generate tokens';
		} finally {
			isGeneratingAiTokens = false;
		}
	}

	/**
	 * Accepts an AI-suggested token by adding it to adhoc input.
	 * Removes the token from the suggestions list.
	 * @param token - The suggested token to accept
	 * @param polarity - Whether to add to positive or negative adhoc
	 */
	function handleAcceptAiToken(token: GeneratedToken, polarity: 'positive' | 'negative') {
		// Format token with weight if not default (1.0)
		const formattedToken =
			Math.abs(token.suggested_weight - 1.0) >= 0.01
				? `(${token.content}:${token.suggested_weight.toFixed(1)})`
				: token.content;

		if (polarity === 'positive') {
			adhocPositive = adhocPositive.trim() ? `${adhocPositive}, ${formattedToken}` : formattedToken;
			aiGeneratedPositive = aiGeneratedPositive.filter((t) => t !== token);
		} else {
			adhocNegative = adhocNegative.trim() ? `${adhocNegative}, ${formattedToken}` : formattedToken;
			aiGeneratedNegative = aiGeneratedNegative.filter((t) => t !== token);
		}
	}

	/**
	 * Dismisses an AI-suggested token without adding it.
	 * @param token - The suggested token to reject
	 * @param polarity - Which list to remove from
	 */
	function handleRejectAiToken(token: GeneratedToken, polarity: 'positive' | 'negative') {
		if (polarity === 'positive') {
			aiGeneratedPositive = aiGeneratedPositive.filter((t) => t !== token);
		} else {
			aiGeneratedNegative = aiGeneratedNegative.filter((t) => t !== token);
		}
	}

	/** Refreshes API key status after successful save */
	async function handleApiKeySaved() {
		apiKeyStatuses = await getApiKeyStatus();
	}
</script>

<div>
	<h1 class="mb-6 text-3xl font-bold text-base-content">Compose Prompt</h1>

	<!-- Row 1: Persona Selection -->
	<Card class="mb-6">
		<div class="flex items-center gap-4">
			<label for="persona-select" class="text-lg font-semibold whitespace-nowrap text-base-content">
				Select Persona
			</label>

			{#if personaStore.isLoading}
				<div class="h-10 w-64 skeleton"></div>
			{:else if personaStore.personas.length === 0}
				<p class="text-sm text-base-content/60">
					No personas available.
					<a href={resolve('/personas/new')} class="text-primary hover:underline">Create one</a>
				</p>
			{:else}
				<select
					id="persona-select"
					class="select-bordered select w-full max-w-xs"
					bind:value={selectedPersonaId}
					onchange={(e) => handlePersonaSelect(e.currentTarget.value)}
				>
					{#each alphabeticallySortedPersonas as persona (persona.id)}
						<option value={persona.id}>{persona.name}</option>
					{/each}
				</select>
			{/if}
		</div>
	</Card>

	<!-- Row 2: Granularity Levels (1/3) + Base Prompt (2/3) -->
	<div class="mb-6 grid grid-cols-3 gap-6">
		<!-- Granularity Selection -->
		<Card>
			<div class="mb-4 flex items-center justify-between">
				<h2 class="text-lg font-semibold text-base-content">Granularity Levels</h2>
				<button type="button" class="btn btn-ghost btn-xs" onclick={toggleAllGranularities}>
					{allGranularitiesSelected ? 'Deselect All' : 'Select All'}
				</button>
			</div>

			<div class="space-y-2">
				{#each tokenStore.granularityLevels as level (level.id)}
					<label class="flex cursor-pointer items-center gap-2">
						<input
							type="checkbox"
							class="checkbox checkbox-sm checkbox-primary"
							checked={selectedGranularityIds.has(level.id)}
							onchange={() => toggleGranularity(level.id)}
						/>
						<span class="text-base-content">{level.name}</span>
					</label>
				{/each}
			</div>

			<div class="mt-4 border-t border-base-300 pt-4">
				<label class="flex cursor-pointer items-center gap-2">
					<input
						type="checkbox"
						class="checkbox checkbox-sm checkbox-primary"
						bind:checked={includeWeights}
					/>
					<span class="text-sm text-base-content">Include token weights</span>
				</label>
			</div>
		</Card>

		<!-- Base Prompt (no copy buttons) - 2/3 width -->
		<Card class="col-span-2">
			<h2 class="mb-4 text-lg font-semibold text-base-content">Base Prompt</h2>

			{#if !selectedPersonaId}
				<p class="py-8 text-center text-base-content/60">Select a persona to see the base prompt</p>
			{:else if isComposing}
				<div class="animate-pulse space-y-4">
					<div class="h-16 rounded bg-base-300"></div>
					<div class="h-16 rounded bg-base-300"></div>
				</div>
			{:else if basePrompt}
				<div class="space-y-4">
					<!-- Positive Base -->
					<div>
						<h3 class="mb-2 text-sm font-medium text-success">Positive</h3>
						<div class="min-h-[50px] border border-success/30 bg-success/10 p-3">
							{#if basePrompt.positive_prompt}
								<p class="text-sm break-words whitespace-pre-wrap text-base-content select-text">
									{basePrompt.positive_prompt}
								</p>
							{:else}
								<p class="text-sm text-base-content/60 italic">No positive tokens</p>
							{/if}
						</div>
					</div>

					<!-- Negative Base -->
					<div>
						<h3 class="mb-2 text-sm font-medium text-error">Negative</h3>
						<div class="min-h-[50px] border border-error/30 bg-error/10 p-3">
							{#if basePrompt.negative_prompt}
								<p class="text-sm break-words whitespace-pre-wrap text-base-content select-text">
									{basePrompt.negative_prompt}
								</p>
							{:else}
								<p class="text-sm text-base-content/60 italic">No negative tokens</p>
							{/if}
						</div>
					</div>
				</div>
			{:else}
				<p class="py-8 text-center text-base-content/60">
					No tokens found for the selected granularity levels
				</p>
			{/if}
		</Card>
	</div>

	<!-- Row 3: AI Token Generation (1/3) + Ad-hoc Tokens (2/3) -->
	<div class="mb-6 grid grid-cols-3 gap-6">
		<!-- AI Token Generation (1/3) -->
		<Card>
			<h2 class="mb-4 text-lg font-semibold text-base-content">AI Token Generation</h2>

			{#if !selectedPersonaId}
				<p class="py-6 text-center text-base-content/60">Select a persona first</p>
			{:else if !hasAiConfigured}
				<!-- Not Configured State -->
				<div class="py-6 text-center">
					<p class="mb-3 text-sm text-base-content/60">
						Configure an AI model in the persona's settings to enable AI token generation.
					</p>
					<a href={resolve(`/personas/${selectedPersonaId}`)} class="btn btn-sm btn-secondary">
						Edit Persona Settings
					</a>
				</div>
			{:else}
				<!-- Configured State -->
				<div class="space-y-4">
					<div>
						<textarea
							id="ai-context"
							bind:value={aiContextDescription}
							rows={3}
							class="textarea-bordered textarea w-full"
							placeholder="Describe the context or action"
						></textarea>
						<p class="mt-1 text-xs text-base-content/60">
							Describe what you want to generate. The AI will suggest relevant tokens.
						</p>
					</div>

					{#if aiGenerationError}
						<div role="alert" class="alert alert-soft alert-error">
							<span>{aiGenerationError}</span>
						</div>
					{/if}

					{#if selectedProviderMissingApiKey}
						<div role="alert" class="alert alert-soft alert-warning">
							<span>
								API key required for {configStore.getProviderById(
									selectedPersona?.ai_provider_id ?? ''
								)?.displayName}.
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

					<div class="flex flex-col gap-2">
						<Button
							onclick={handleGenerateAiTokens}
							disabled={isGeneratingAiTokens ||
								!aiContextDescription.trim() ||
								selectedProviderMissingApiKey}
						>
							{isGeneratingAiTokens ? 'Generating...' : 'Generate Tokens'}
						</Button>

						<span class="text-xs text-base-content/60">
							Using: {configStore.getProviderById(selectedPersona?.ai_provider_id ?? '')
								?.displayName ?? 'Unknown'} / {selectedPersona?.ai_model_id}
						</span>
					</div>
				</div>
			{/if}
		</Card>

		<!-- Ad-hoc Tokens (2/3) -->
		<Card class="col-span-2">
			<h2 class="mb-4 text-lg font-semibold text-base-content">Ad-hoc Tokens</h2>

			<div class="space-y-4">
				<!-- Extra Positive with Total Token Count -->
				<div>
					<div class="mb-1 flex items-center justify-between">
						<label for="adhoc-positive" class="text-sm font-medium text-base-content">
							Extra Positive
						</label>
						<div class="flex items-center gap-2">
							<span class="text-xs text-base-content/60">Total:</span>
							<TokenCountBadge
								tokenCount={positiveTokenCount}
								isLoading={isCountingTokens}
								showBar={false}
							/>
						</div>
					</div>
					<input
						type="text"
						id="adhoc-positive"
						bind:value={adhocPositive}
						placeholder="masterpiece, best quality"
						class="input-bordered input w-full"
					/>
				</div>

				<!-- Extra Negative with Total Token Count -->
				<div>
					<div class="mb-1 flex items-center justify-between">
						<label for="adhoc-negative" class="text-sm font-medium text-base-content">
							Extra Negative
						</label>
						<div class="flex items-center gap-2">
							<span class="text-xs text-base-content/60">Total:</span>
							<TokenCountBadge
								tokenCount={negativeTokenCount}
								isLoading={isCountingTokens}
								showBar={false}
							/>
						</div>
					</div>
					<input
						type="text"
						id="adhoc-negative"
						bind:value={adhocNegative}
						placeholder="low quality, bad anatomy"
						class="input-bordered input w-full"
					/>
				</div>

				<!-- AI Generated Suggestions -->
				{#if aiGeneratedPositive.length > 0 || aiGeneratedNegative.length > 0}
					<div class="mt-4 border-t border-base-300 pt-4">
						<h3 class="mb-2 text-sm font-medium text-base-content">AI Suggestions</h3>

						{#if aiGeneratedPositive.length > 0}
							<div class="mb-3">
								<span class="text-xs font-medium text-success">Positive:</span>
								<div class="mt-1 flex flex-wrap gap-2">
									{#each aiGeneratedPositive as token (token.content)}
										<div
											class="badge gap-1 badge-outline badge-success"
											title={token.rationale ?? ''}
										>
											<span>{token.content}</span>
											{#if Math.abs(token.suggested_weight - 1.0) >= 0.01}
												<span class="text-xs opacity-70">({token.suggested_weight.toFixed(1)})</span
												>
											{/if}
											<button
												type="button"
												class="btn px-1 btn-ghost btn-xs"
												onclick={() => handleAcceptAiToken(token, 'positive')}
												title="Add to Extra Positive">+</button
											>
											<button
												type="button"
												class="btn px-1 btn-ghost btn-xs"
												onclick={() => handleRejectAiToken(token, 'positive')}
												title="Dismiss">&times;</button
											>
										</div>
									{/each}
								</div>
							</div>
						{/if}

						{#if aiGeneratedNegative.length > 0}
							<div>
								<span class="text-xs font-medium text-error">Negative:</span>
								<div class="mt-1 flex flex-wrap gap-2">
									{#each aiGeneratedNegative as token (token.content)}
										<div
											class="badge gap-1 badge-outline badge-error"
											title={token.rationale ?? ''}
										>
											<span>{token.content}</span>
											{#if Math.abs(token.suggested_weight - 1.0) >= 0.01}
												<span class="text-xs opacity-70">({token.suggested_weight.toFixed(1)})</span
												>
											{/if}
											<button
												type="button"
												class="btn px-1 btn-ghost btn-xs"
												onclick={() => handleAcceptAiToken(token, 'negative')}
												title="Add to Extra Negative">+</button
											>
											<button
												type="button"
												class="btn px-1 btn-ghost btn-xs"
												onclick={() => handleRejectAiToken(token, 'negative')}
												title="Dismiss">&times;</button
											>
										</div>
									{/each}
								</div>
							</div>
						{/if}
					</div>
				{/if}
			</div>
		</Card>
	</div>

	<!-- Row 4: Final Prompts (Full Width) -->
	<Card>
		<div class="mb-4 flex items-center justify-between">
			<h2 class="text-lg font-semibold text-base-content">Final Prompts</h2>
			{#if composedPrompt}
				<Button variant="secondary" size="sm" onclick={handleCopyBoth}>
					{copySuccess === 'both' ? 'Copied!' : 'Copy Both'}
				</Button>
			{/if}
		</div>

		{#if !selectedPersonaId}
			<p class="py-8 text-center text-base-content/60">Select a persona to compose a prompt</p>
		{:else if isComposing}
			<div class="animate-pulse space-y-4">
				<div class="h-20 rounded bg-base-300"></div>
				<div class="h-20 rounded bg-base-300"></div>
			</div>
		{:else if composedPrompt}
			<div class="space-y-4">
				<!-- Positive Prompt -->
				<div>
					<div class="mb-2 flex items-center justify-between">
						<div class="flex items-center gap-2">
							<h3 class="text-sm font-medium text-success">Positive Prompt</h3>
							<TokenCountBadge tokenCount={positiveTokenCount} isLoading={isCountingTokens} />
						</div>
						<button type="button" class="btn btn-ghost btn-xs" onclick={handleCopyPositive}>
							{copySuccess === 'positive' ? 'Copied!' : 'Copy'}
						</button>
					</div>
					<div class="min-h-[60px] border border-success/30 bg-success/10 p-4">
						{#if composedPrompt.positive_prompt}
							<p class="text-sm break-words whitespace-pre-wrap text-base-content select-text">
								{composedPrompt.positive_prompt}
							</p>
						{:else}
							<p class="text-sm text-base-content/60 italic">No positive tokens</p>
						{/if}
					</div>
				</div>

				<!-- Negative Prompt -->
				<div>
					<div class="mb-2 flex items-center justify-between">
						<div class="flex items-center gap-2">
							<h3 class="text-sm font-medium text-error">Negative Prompt</h3>
							<TokenCountBadge tokenCount={negativeTokenCount} isLoading={isCountingTokens} />
						</div>
						<button type="button" class="btn btn-ghost btn-xs" onclick={handleCopyNegative}>
							{copySuccess === 'negative' ? 'Copied!' : 'Copy'}
						</button>
					</div>
					<div class="min-h-[60px] border border-error/30 bg-error/10 p-4">
						{#if composedPrompt.negative_prompt}
							<p class="text-sm break-words whitespace-pre-wrap text-base-content select-text">
								{composedPrompt.negative_prompt}
							</p>
						{:else}
							<p class="text-sm text-base-content/60 italic">No negative tokens</p>
						{/if}
					</div>
				</div>
			</div>
		{:else}
			<p class="py-8 text-center text-base-content/60">
				No tokens found for the selected granularity levels
			</p>
		{/if}
	</Card>
</div>

{#if selectedPersona?.ai_provider_id}
	{@const providerId = selectedPersona.ai_provider_id}
	{@const provider = configStore.getProviderById(providerId)}
	<ApiKeyModal
		bind:open={showApiKeyModal}
		{providerId}
		providerDisplayName={provider?.displayName ?? providerId}
		hasExistingKey={false}
		onsave={handleApiKeySaved}
	/>
{/if}
