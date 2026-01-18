<!--
@component
Spinner - Loading indicator component.

Displays an animated spinner using daisyUI's loading styles.
Use during async operations to indicate activity.
-->
<script lang="ts">
	/**
	 * @property size - Spinner size: xs, sm, md, or lg
	 * @property class - Additional CSS classes
	 */
	interface Props {
		size?: 'xs' | 'sm' | 'md' | 'lg';
		class?: string;
	}

	let { size = 'md', class: className = '' }: Props = $props();

	/** Maps size prop to daisyUI loading size classes */
	const sizeClasses: Record<string, string> = {
		xs: 'loading-xs',
		sm: 'loading-sm',
		md: 'loading-md',
		lg: 'loading-lg'
	};
</script>

<span class="loading loading-spinner {sizeClasses[size]} {className}"></span>
