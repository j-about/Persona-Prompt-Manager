import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	// Tauri expects a fixed port, fail if that port is not available
	server: {
		port: 5173,
		strictPort: true
	},
	// Enable env prefix for Tauri
	envPrefix: ['VITE_', 'TAURI_']
});
