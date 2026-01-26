/**
 * Prompt composition types - TypeScript equivalents of Rust domain types
 */

/** Position for ad-hoc tokens in the composed prompt */
export type AdhocPosition = 'beginning' | 'end';

/** A composed prompt ready for use in image generation */
export interface ComposedPrompt {
	positive_prompt: string;
	negative_prompt: string;
	positive_token_count: number;
	negative_token_count: number;
	breakdown: PromptBreakdown;
}

/** Breakdown of the prompt by granularity level */
export interface PromptBreakdown {
	sections: GranularitySection[];
}

/** Tokens grouped by a single granularity level */
export interface GranularitySection {
	granularity_id: string;
	granularity_name: string;
	positive_tokens: string[];
	negative_tokens: string[];
}

/** Options for prompt composition */
export interface CompositionOptions {
	/** Whether to include weight modifiers in the output */
	include_weights?: boolean;
	/** Separator between tokens */
	separator?: string;
	/** Granularity levels to include (in order) */
	granularity_ids?: string[];
	/** Ad-hoc positive tokens to append */
	adhoc_positive?: string | null;
	/** Ad-hoc negative tokens to append */
	adhoc_negative?: string | null;
	/** Where to place ad-hoc tokens */
	adhoc_position?: AdhocPosition;
}
