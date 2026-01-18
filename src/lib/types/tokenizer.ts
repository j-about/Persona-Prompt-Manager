/**
 * Tokenizer types - TypeScript equivalents of Rust tokenizer types
 *
 * Note: Model configurations are fetched from the Rust backend via getKnownModels().
 * Rust is the single source of truth for model/tokenizer mappings.
 */

/** Token count result with detailed breakdown */
export interface TokenCount {
	/** Number of tokens in the text */
	count: number;
	/** Maximum allowed tokens for the model */
	max_tokens: number;
	/** Usable tokens (excluding special tokens) */
	usable_tokens: number;
	/** Whether the text exceeds the token limit */
	exceeds_limit: boolean;
	/** Percentage of limit used (0-100+) */
	usage_percent: number;
	/** The model this count is for */
	model_id: string;
	/** The tokenizer used */
	tokenizer_id: string;
}

/** Information about the tokenizer for a model (fetched from Rust backend) */
export interface TokenizerInfo {
	/** Model ID (default is defined in backend, see `getDefaultImageModelId()`) */
	model_id: string;
	/** Tokenizer ID from HuggingFace (e.g., "openai/clip-vit-large-patch14") */
	tokenizer_id: string;
	/** Whether the tokenizer is available/loaded */
	available: boolean;
	/** Maximum tokens for the model */
	max_tokens: number;
	/** Usable tokens (excluding special tokens like BOS/EOS) */
	usable_tokens: number;
}
