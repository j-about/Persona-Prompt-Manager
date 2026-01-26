<!--
@component
Button - Styled button component with variant and size options.

Uses daisyUI button classes for consistent styling across the app.
Extends native button attributes for full HTML compatibility.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	/**
	 * @property variant - Visual style: primary, secondary, danger, or ghost
	 * @property size - Button size: sm, md, or lg
	 * @property disabled - Disables the button
	 * @property children - Button content (text, icons, etc.)
	 */
	interface Props extends HTMLButtonAttributes {
		variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
		size?: 'sm' | 'md' | 'lg';
		children: Snippet;
	}

	let {
		variant = 'primary',
		size = 'md',
		disabled = false,
		class: className = '',
		children,
		...rest
	}: Props = $props();

	// daisyUI button variant classes
	const variantClasses: Record<string, string> = {
		primary: 'btn-primary',
		secondary: 'btn-secondary',
		danger: 'btn-error',
		ghost: 'btn-ghost'
	};

	// daisyUI button size classes
	const sizeClasses: Record<string, string> = {
		sm: 'btn-sm',
		md: 'btn-md',
		lg: 'btn-lg'
	};

	const classes = $derived(`btn ${variantClasses[variant]} ${sizeClasses[size]} ${className}`);
</script>

<button class={classes} {disabled} {...rest}>
	{@render children()}
</button>
