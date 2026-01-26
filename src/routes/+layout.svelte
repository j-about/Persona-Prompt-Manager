<!--
@component
Root Layout - Main application shell with navigation sidebar.

Provides the app-wide layout including sidebar navigation, version display,
toast notifications, and donation popup. Also handles initialization of
config store and credential store availability check on Linux.

@route - All routes
-->
<script lang="ts">
	import '../app.css';
	import { onMount, onDestroy } from 'svelte';
	import { page } from '$app/stores';
	import { resolve } from '$app/paths';
	import { Toast, DonationPopup } from '$lib/components/ui';
	import { checkCredentialStore } from '$lib/services/settings';
	import { configStore, donationStore } from '$lib/stores';
	import { getVersion } from '@tauri-apps/api/app';
	import { exit } from '@tauri-apps/plugin-process';
	import { type as osType } from '@tauri-apps/plugin-os';
	import { getCurrentWindow } from '@tauri-apps/api/window';

	interface Props {
		children: import('svelte').Snippet;
	}

	let { children }: Props = $props();

	let credentialStoreUnavailable = $state(false);
	let checkingCredentialStore = $state(true);
	let appVersion = $state('');
	let unlistenCloseRequest: (() => void) | null = null;

	const navItems = [
		{ href: '/' as const, label: 'Home' },
		{ href: '/personas' as const, label: 'Personas' },
		{ href: '/compose' as const, label: 'Compose' },
		{ href: '/settings' as const, label: 'Settings' }
	];

	onMount(async () => {
		try {
			const platform = await osType();
			if (platform === 'linux') {
				const available = await checkCredentialStore();
				credentialStoreUnavailable = !available;
			}

			// Initialize configuration from Rust backend (single source of truth)
			await configStore.initialize();

			// Get app version from Tauri (reads from tauri.conf.json)
			appVersion = await getVersion();

			// Register window close request handler
			unlistenCloseRequest = await getCurrentWindow().onCloseRequested(async (event) => {
				event.preventDefault();
				donationStore.open();
			});
		} catch (e) {
			console.error('Failed to initialize app:', e);
		} finally {
			checkingCredentialStore = false;
		}
	});

	onDestroy(() => {
		if (unlistenCloseRequest) {
			unlistenCloseRequest();
		}
	});

	async function handleQuit() {
		await exit(1);
	}
</script>

{#if credentialStoreUnavailable}
	<!-- Blocking modal for missing Secret Service -->
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-base-300/90">
		<div class="card mx-4 max-w-lg bg-base-100 shadow-2xl">
			<div class="card-body">
				<h2 class="card-title text-error">Secure Storage Unavailable</h2>
				<p class="text-base-content/70">
					This application requires a system secret service to securely store API keys. No
					compatible service was detected.
				</p>

				<div class="divider">How to resolve</div>

				<div class="space-y-3 text-sm">
					<p>
						<span class="font-semibold">GNOME-based systems:</span> Install
						<code class="badge badge-ghost badge-sm">gnome-keyring</code>
					</p>
					<p>
						<span class="font-semibold">KDE-based systems:</span> Install
						<code class="badge badge-ghost badge-sm">kwallet</code>
					</p>
					<p class="mt-4 text-xs text-base-content/60">
						After installation, log out and log back in to activate the keyring service, then
						restart this application.
					</p>
				</div>

				<div class="mt-6 card-actions justify-end">
					<button class="btn btn-error" onclick={handleQuit}>Quit Application</button>
				</div>
			</div>
		</div>
	</div>
{:else if checkingCredentialStore}
	<!-- Loading state while checking -->
	<div class="flex min-h-screen items-center justify-center bg-base-200">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else}
	<!-- Normal app content -->
	<div class="flex min-h-screen">
		<nav
			class="fixed flex h-screen w-64 flex-col overflow-y-auto bg-base-300 p-4 text-base-content"
		>
			<img src="/logo.png" alt="Persona Prompt Manager" class="mb-8 w-full" />
			<ul class="menu w-full flex-1 menu-lg p-0">
				{#each navItems as item (item.href)}
					<li>
						<a
							href={resolve(item.href)}
							class="justify-center {$page.url.pathname === item.href ||
							($page.url.pathname.startsWith(item.href) && item.href !== '/')
								? 'active'
								: ''}"
						>
							{item.label}
						</a>
					</li>
				{/each}
			</ul>
			<div class="flex items-center justify-between text-sm text-base-content/60">
				<a
					href={resolve('/about')}
					class="transition-colors hover:text-base-content"
					class:text-base-content={$page.url.pathname === '/about'}
				>
					About
				</a>
				<span>v{appVersion}</span>
			</div>
		</nav>
		<main class="ml-64 flex-1 bg-base-200 p-8">
			{@render children()}
		</main>
	</div>
	<Toast />
{/if}

{#if donationStore.isOpen}
	<DonationPopup />
{/if}
