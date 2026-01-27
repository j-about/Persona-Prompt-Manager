<!--
@component
Badge - Inline label component for status or category display.

Supports semantic variants (positive/negative for tokens, info/warning).
-->
<script lang="ts" module>
	/**
	 * Available badge color variants.
	 * - Polarity variants: positive (green), negative (red)
	 * - Status variants: info (blue), warning (orange), default (gray)
	 */
	export type BadgeVariant = 'default' | 'positive' | 'negative' | 'info' | 'warning';
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';

	/**
	 * @property variant - Color scheme for the badge
	 * @property size - Badge size: sm or md
	 * @property class - Additional CSS classes
	 * @property children - Badge label content
	 */
	interface Props {
		variant?: BadgeVariant;
		size?: 'sm' | 'md';
		class?: string;
		children: Snippet;
	}

	let { variant = 'default', size = 'md', class: className = '', children }: Props = $props();

	/** Maps variant to daisyUI badge classes */
	const variantClasses: Record<BadgeVariant, string> = {
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
