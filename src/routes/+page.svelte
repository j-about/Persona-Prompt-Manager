<script lang="ts">
	import { resolve } from '$app/paths';
	// Home page - dashboard view
</script>

<div>
	<h1 class="mb-6 text-3xl font-bold text-base-content">Welcome to Persona Prompt Manager</h1>

	<p class="mb-8 text-lg text-base-content/70">
		Manage your AI image generation personas with precision and consistency.
	</p>

	<div class="grid grid-cols-2 gap-6">
		<!-- Quick Actions -->
		<a
			href={resolve('/personas/new')}
			class="block bg-base-100 p-6 shadow-md transition-shadow hover:shadow-lg"
		>
			<h2 class="mb-2 text-xl font-semibold text-base-content">Create Persona</h2>
			<p class="text-base-content/60">
				Start a new character profile with organized token management.
			</p>
		</a>

		<a
			href={resolve('/personas')}
			class="block bg-base-100 p-6 shadow-md transition-shadow hover:shadow-lg"
		>
			<h2 class="mb-2 text-xl font-semibold text-base-content">View Personas</h2>
			<p class="text-base-content/60">Browse and manage your existing character profiles.</p>
		</a>

		<a
			href={resolve('/compose')}
			class="block bg-base-100 p-6 shadow-md transition-shadow hover:shadow-lg"
		>
			<h2 class="mb-2 text-xl font-semibold text-base-content">Compose Prompt</h2>
			<p class="text-base-content/60">
				Build prompts by selecting persona elements and granularity levels.
			</p>
		</a>

		<a
			href={resolve('/settings')}
			class="block bg-base-100 p-6 shadow-md transition-shadow hover:shadow-lg"
		>
			<h2 class="mb-2 text-xl font-semibold text-base-content">Settings</h2>
			<p class="text-base-content/60">
				Manage API keys for AI providers and import/export your data.
			</p>
		</a>
	</div>
</div>
