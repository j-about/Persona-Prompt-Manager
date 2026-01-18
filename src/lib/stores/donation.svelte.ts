/**
 * Donation popup store - Svelte 5 runes-based state management
 */

function createDonationStore() {
	let isOpen = $state(false);

	function open(): void {
		isOpen = true;
	}

	function close(): void {
		isOpen = false;
	}

	return {
		get isOpen() {
			return isOpen;
		},
		open,
		close
	};
}

export const donationStore = createDonationStore();
