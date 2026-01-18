/**
 * Persona store - Svelte 5 runes-based state management
 *
 * Uses Svelte 5's $state and $derived runes for reactive state management.
 */

import * as personaService from '$lib/services/persona';
import type { Persona, CreatePersonaRequest, UpdatePersonaRequest } from '$lib/types';

/** Create a reactive persona store */
function createPersonaStore() {
	// Reactive state using $state rune
	let personas = $state<Persona[]>([]);
	let selectedPersona = $state<Persona | null>(null);
	let isLoading = $state(false);
	let error = $state<string | null>(null);

	// Derived values
	const sortedPersonas = $derived(
		[...personas].sort((a, b) => {
			// Sort by updated_at descending (most recent first)
			return new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime();
		})
	);

	const personaCount = $derived(personas.length);

	// Actions
	async function loadAll(): Promise<void> {
		isLoading = true;
		error = null;
		try {
			personas = await personaService.listPersonas();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load personas';
			console.error('Failed to load personas:', err);
		} finally {
			isLoading = false;
		}
	}

	async function loadById(id: string): Promise<Persona | null> {
		isLoading = true;
		error = null;
		try {
			const persona = await personaService.getPersona(id);
			selectedPersona = persona;
			return persona;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load persona';
			console.error('Failed to load persona:', err);
			return null;
		} finally {
			isLoading = false;
		}
	}

	async function search(query: string): Promise<Persona[]> {
		isLoading = true;
		error = null;
		try {
			const results = await personaService.searchPersonas(query);
			personas = results;
			return results;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to search personas';
			console.error('Failed to search personas:', err);
			return [];
		} finally {
			isLoading = false;
		}
	}

	async function create(request: CreatePersonaRequest): Promise<Persona | null> {
		isLoading = true;
		error = null;
		try {
			const newPersona = await personaService.createPersona(request);
			personas = [...personas, newPersona];
			selectedPersona = newPersona;
			return newPersona;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to create persona';
			console.error('Failed to create persona:', err);
			return null;
		} finally {
			isLoading = false;
		}
	}

	async function update(id: string, request: UpdatePersonaRequest): Promise<Persona | null> {
		isLoading = true;
		error = null;
		try {
			const updatedPersona = await personaService.updatePersona(id, request);
			personas = personas.map((p) => (p.id === id ? updatedPersona : p));
			if (selectedPersona?.id === id) {
				selectedPersona = updatedPersona;
			}
			return updatedPersona;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to update persona';
			console.error('Failed to update persona:', err);
			return null;
		} finally {
			isLoading = false;
		}
	}

	async function remove(id: string): Promise<boolean> {
		isLoading = true;
		error = null;
		try {
			await personaService.deletePersona(id);
			personas = personas.filter((p) => p.id !== id);
			if (selectedPersona?.id === id) {
				selectedPersona = null;
			}
			return true;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to delete persona';
			console.error('Failed to delete persona:', err);
			return false;
		} finally {
			isLoading = false;
		}
	}

	async function duplicate(id: string, newName?: string): Promise<Persona | null> {
		isLoading = true;
		error = null;
		try {
			const duplicated = await personaService.duplicatePersona(id, newName);
			personas = [...personas, duplicated];
			return duplicated;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to duplicate persona';
			console.error('Failed to duplicate persona:', err);
			return null;
		} finally {
			isLoading = false;
		}
	}

	function select(persona: Persona | null): void {
		selectedPersona = persona;
	}

	function clearError(): void {
		error = null;
	}

	return {
		// State getters (reactive via $derived where needed)
		get personas() {
			return personas;
		},
		get sortedPersonas() {
			return sortedPersonas;
		},
		get selectedPersona() {
			return selectedPersona;
		},
		get isLoading() {
			return isLoading;
		},
		get error() {
			return error;
		},
		get count() {
			return personaCount;
		},

		// Actions
		loadAll,
		loadById,
		search,
		create,
		update,
		remove,
		duplicate,
		select,
		clearError
	};
}

// Export a singleton instance
export const personaStore = createPersonaStore();
