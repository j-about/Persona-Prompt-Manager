<!--
@component
Badge - Inline label component for status or category display.

Supports semantic variants (positive/negative for tokens, info/warning)
with soft styling for colored variants.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	/**
	 * @property variant - Color scheme: positive (green), negative (red), info, warning, or default
	 * @property size - Badge size: sm or md
	 * @property class - Additional CSS classes
	 * @property children - Badge label content
	 */
	interface Props {
		variant?: 'default' | 'positive' | 'negative' | 'info' | 'warning';
		size?: 'sm' | 'md';
		class?: string;
		children: Snippet;
	}

	let { variant = 'default', size = 'md', class: className = '', children }: Props = $props();

	/** Maps variant to daisyUI badge classes */
	const variantClasses: Record<string, string> = {
		default: 'badge-neutral',
		positive: 'badge-success badge-soft',
		negative: 'badge-error badge-soft',
		info: 'badge-info badge-soft',
		warning: 'badge-warning badge-soft'
	};

	/** Maps size to daisyUI size classes */
	const sizeClasses: Record<string, string> = {
		sm: 'badge-sm',
		md: 'badge-md'
	};

	const classes = $derived(`badge ${variantClasses[variant]} ${sizeClasses[size]} ${className}`);
</script>

<span class={classes}>
	{@render children()}
</span>
