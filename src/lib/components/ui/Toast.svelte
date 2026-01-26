<!--
@component
Toast - Notification display component for showing temporary messages.

Renders toast notifications from the toast store in the top-right corner.
Supports success, error, info, and warning types with appropriate styling.
Each toast includes a dismiss button.
-->
<script lang="ts">
	import { toastStore, type Toast } from '$lib/stores/toast.svelte';

	const typeStyles: Record<Toast['type'], string> = {
		success: 'alert-success',
		error: 'alert-error',
		info: 'alert-info',
		warning: 'alert-warning'
	};

	const typeIcons: Record<Toast['type'], string> = {
		success: 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z',
		error: 'M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z',
		info: 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z',
		warning:
			'M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z'
	};
</script>

<div class="toast toast-end toast-top z-50">
	{#each toastStore.toasts as toast (toast.id)}
		<div class="alert {typeStyles[toast.type]} shadow-lg">
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="h-5 w-5 shrink-0 stroke-current"
				fill="none"
				viewBox="0 0 24 24"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d={typeIcons[toast.type]}
				/>
			</svg>
			<span class="select-text">{toast.message}</span>
			<button
				type="button"
				class="btn btn-ghost btn-xs"
				onclick={() => toastStore.dismiss(toast.id)}
				aria-label="Dismiss"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-4 w-4"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M6 18L18 6M6 6l12 12"
					/>
				</svg>
			</button>
		</div>
	{/each}
</div>
