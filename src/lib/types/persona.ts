/**
 * Persona types - TypeScript equivalents of Rust domain types
 */

import type { ISODateString, UUID } from './common';

/** A Persona represents a complete fictional character profile */
export interface Persona {
	id: UUID;
	name: string;
	description: string | null;
	tags: string[];
	/** Optional AI provider for token generation (required for AI features) */
	ai_provider_id: string | null;
	/** Optional AI model for token generation (required for AI features) */
	ai_model_id: string | null;
	/** Custom instructions for AI token generation */
	ai_instructions: string | null;
	created_at: ISODateString;
	updated_at: ISODateString;
}

/** Generation parameters for image generation */
export interface GenerationParams {
	persona_id: UUID;
	model_id: string;
	seed: number;
	steps: number;
	cfg_scale: number;
	sampler: string | null;
	/** Scheduler algorithm (e.g., "karras", "exponential", "normal") */
	scheduler: string | null;
}

/** Request to create a new persona */
export interface CreatePersonaRequest {
	name: string;
	description?: string | null;
	tags?: string[];
}

/** Request to update an existing persona */
export interface UpdatePersonaRequest {
	name?: string;
	description?: string | null;
	tags?: string[];
	ai_provider_id?: string | null;
	ai_model_id?: string | null;
	ai_instructions?: string | null;
}
