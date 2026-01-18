<script lang="ts">
	import { onMount } from 'svelte';
	import { open } from '@tauri-apps/plugin-shell';
	import { exit } from '@tauri-apps/plugin-process';
	import { Button } from '$lib/components/ui';
	import { donationStore } from '$lib/stores';

	const STRIPE_LINKS = {
		eur3: 'https://buy.stripe.com/9B6eVceEw7IHe5igqo6Ri04',
		eur5: 'https://buy.stripe.com/8x2bJ07c42onbXa8XW6Ri05',
		eur10: 'https://buy.stripe.com/cNi4gy0NG8MLf9mca86Ri06',
		custom: 'https://buy.stripe.com/9B65kC0NGe754uIfmk6Ri07'
	};

	let dialogRef: HTMLDialogElement;

	onMount(() => {
		dialogRef?.showModal();
	});

	async function handleDonate(amount: keyof typeof STRIPE_LINKS) {
		await open(STRIPE_LINKS[amount]);
	}

	function handleReturnToApp() {
		dialogRef?.close();
		donationStore.close();
	}

	async function handleExit() {
		await exit(0);
	}
</script>

<dialog bind:this={dialogRef} class="modal" onclose={handleReturnToApp}>
	<div class="modal-box">
		<h3 class="text-lg font-bold">Support Persona Prompt Manager</h3>

		<p class="py-4 text-base-content/70">
			If you find this app useful, consider supporting its development with a small donation.
		</p>

		<div class="mb-6 grid grid-cols-4 gap-2">
			<Button variant="secondary" onclick={() => handleDonate('eur3')}>
				{@render amount(3)}
			</Button>
			<Button variant="secondary" onclick={() => handleDonate('eur5')}>
				{@render amount(5)}
			</Button>
			<Button variant="secondary" onclick={() => handleDonate('eur10')}>
				{@render amount(10)}
			</Button>
			<Button variant="ghost" onclick={() => handleDonate('custom')}>Custom</Button>
		</div>

		<div class="divider">or</div>

		<div class="modal-action justify-between">
			<Button variant="ghost" onclick={handleReturnToApp}>Return to App</Button>
			<Button variant="danger" onclick={handleExit}>Exit Application</Button>
		</div>
	</div>
	<form method="dialog" class="modal-backdrop">
		<button onclick={handleReturnToApp}>close</button>
	</form>
</dialog>

{#snippet amount(value: number)}
	<span class="text-lg">{value}</span>
{/snippet}
