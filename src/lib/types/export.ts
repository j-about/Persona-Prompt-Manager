/**
 * Export/Import types - TypeScript equivalents of Rust export types
 */

import type { Persona, GenerationParams } from './persona';
import type { Token, GranularityLevel } from './token';

/** Export format for a single persona with all its data */
export interface PersonaExport {
	/** Export format version for compatibility */
	version: string;
	/** Export timestamp */
	exported_at: string;
	/** The persona data */
	persona: Persona;
	/** Generation parameters */
	generation_params: GenerationParams;
	/** All tokens for this persona */
	tokens: Token[];
	/** Granularity levels used */
	granularity_levels: GranularityLevel[];
}

/** Export format for multiple personas (bulk export) */
export interface BulkExport {
	/** Export format version */
	version: string;
	/** Export timestamp */
	exported_at: string;
	/** Application name/identifier */
	app: string;
	/** All personas with their data */
	personas: PersonaExport[];
}

/** Import result for a single persona */
export interface ImportResult {
	/** Whether the import was successful */
	success: boolean;
	/** The imported persona (if successful) */
	persona?: Persona | null;
	/** Number of tokens imported */
	tokens_imported: number;
	/** Warning messages (non-fatal issues) */
	warnings: string[];
	/** Error message (if failed) */
	error?: string | null;
}

/** Strategy for handling import conflicts */
export type ImportConflictStrategy = 'skip' | 'rename' | 'replace';

/** Options for import behavior */
export interface ImportOptions {
	/** What to do if a persona with the same name exists */
	on_conflict: ImportConflictStrategy;
	/** Whether to import granularity levels */
	import_granularities: boolean;
}
