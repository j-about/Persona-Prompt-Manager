<!--
@component
About Page - Application information and credits.

Displays the app logo, name, version, description, and license information.

@route /about
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { getVersion } from '@tauri-apps/api/app';

	const CURRENT_YEAR = new Date().getFullYear();
	let appVersion = $state('');

	onMount(async () => {
		appVersion = await getVersion();
	});
</script>

<div>
	<h1 class="mb-6 text-3xl font-bold text-base-content">About</h1>

	<div class="flex flex-col items-center py-8 text-center">
		<img src="/icon.png" alt="" class="mb-6 h-32 w-32" />

		<h2 class="text-2xl font-bold text-base-content">Persona Prompt Manager</h2>
		<p class="mt-1 text-base-content/60">Version {appVersion}</p>

		<p class="mt-6 max-w-md text-base-content/70">
			A cross-platform desktop application built in Rust that enables digital artists and AI image
			generation practitioners to create, manage, and compose prompts for fictional character
			generation with consistency and precision.
		</p>

		<div class="mt-8 space-y-1 text-sm text-base-content/50">
			<p>© {CURRENT_YEAR} Jonathan About</p>
			<p>MIT License</p>
		</div>
	</div>
</div>
