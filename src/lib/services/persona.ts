/**
 * Persona service - Tauri IPC wrapper for persona operations
 */

import { tauriInvoke } from './tauri';
import type {
	Persona,
	CreatePersonaRequest,
	UpdatePersonaRequest,
	GenerationParams
} from '$lib/types';

/** Create a new persona */
export async function createPersona(request: CreatePersonaRequest): Promise<Persona> {
	return tauriInvoke<Persona>('create_persona', { request });
}

/** Get a persona by ID */
export async function getPersona(id: string): Promise<Persona> {
	return tauriInvoke<Persona>('get_persona_by_id', { id });
}

/** Get all personas */
export async function listPersonas(): Promise<Persona[]> {
	return tauriInvoke<Persona[]>('list_personas');
}

/** Update a persona */
export async function updatePersona(id: string, request: UpdatePersonaRequest): Promise<Persona> {
	return tauriInvoke<Persona>('update_persona', { id, request });
}

/** Delete a persona */
export async function deletePersona(id: string): Promise<void> {
	return tauriInvoke<void>('delete_persona', { id });
}

/** Get generation parameters for a persona */
export async function getGenerationParams(personaId: string): Promise<GenerationParams> {
	return tauriInvoke<GenerationParams>('get_persona_generation_params', { personaId });
}

/** Update generation parameters for a persona */
export async function updateGenerationParams(params: GenerationParams): Promise<void> {
	return tauriInvoke<void>('update_generation_params', { params });
}

/** Duplicate a persona */
export async function duplicatePersona(id: string, newName?: string): Promise<Persona> {
	return tauriInvoke<Persona>('duplicate_persona', { id, newName });
}
