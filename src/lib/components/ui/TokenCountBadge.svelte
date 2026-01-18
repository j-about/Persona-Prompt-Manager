<script lang="ts">
	import type { TokenCount } from '$lib/types';

	interface Props {
		tokenCount: TokenCount | null;
		isLoading?: boolean;
		showBar?: boolean;
		class?: string;
	}

	let { tokenCount, isLoading = false, showBar = true, class: className = '' }: Props = $props();

	const statusColor = $derived(() => {
		if (!tokenCount) return 'text-base-content/50';
		if (tokenCount.exceeds_limit) return 'text-error';
		if (tokenCount.usage_percent > 80) return 'text-warning';
		return 'text-success';
	});

	const progressColor = $derived(() => {
		if (!tokenCount) return '';
		if (tokenCount.exceeds_limit) return 'progress-error';
		if (tokenCount.usage_percent > 80) return 'progress-warning';
		return 'progress-success';
	});

	const progressValue = $derived(() => {
		if (!tokenCount) return 0;
		return Math.min(tokenCount.usage_percent, 100);
	});
</script>

<div class="flex items-center gap-2 {className}">
	{#if isLoading}
		<div class="flex animate-pulse items-center gap-2">
			<div class="h-4 w-12 skeleton"></div>
			{#if showBar}
				<div class="h-1.5 w-20 skeleton rounded-full"></div>
			{/if}
		</div>
	{:else if tokenCount}
		<span class="font-mono text-sm {statusColor()}">
			{tokenCount.count}/{tokenCount.usable_tokens}
		</span>
		{#if showBar}
			<progress class="progress {progressColor()} h-1.5 w-20" value={progressValue()} max="100"
			></progress>
		{/if}
		{#if tokenCount.exceeds_limit}
			<span class="text-xs text-error"> (exceeds limit!) </span>
		{/if}
	{:else}
		<span class="text-sm text-base-content/50">--/--</span>
	{/if}
</div>
