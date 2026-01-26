/**
 * Toast store - Svelte 5 runes-based state management for notifications
 */

export type ToastType = 'success' | 'error' | 'info' | 'warning';

export interface Toast {
	id: string;
	message: string;
	type: ToastType;
	duration: number;
}

function createToastStore() {
	let toasts = $state<Toast[]>([]);

	function show(message: string, type: ToastType = 'info', duration = 5000): string {
		const id = crypto.randomUUID();
		const toast: Toast = { id, message, type, duration };
		toasts = [...toasts, toast];

		if (duration > 0) {
			setTimeout(() => dismiss(id), duration);
		}

		return id;
	}

	function dismiss(id: string): void {
		toasts = toasts.filter((t) => t.id !== id);
	}

	function success(message: string, duration = 5000): string {
		return show(message, 'success', duration);
	}

	return {
		get toasts() {
			return toasts;
		},
		show,
		dismiss,
		success
	};
}

export const toastStore = createToastStore();
