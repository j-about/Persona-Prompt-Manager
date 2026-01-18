/**
 * AI-related types - TypeScript equivalents of Rust AI types
 */

/**
 * AI provider identifier string.
 *
 * Valid provider IDs are fetched from the Rust backend via `getAiProviderIds()`,
 * ensuring a single source of truth. The backend defines the canonical list
 * in `src-tauri/src/domain/ai.rs`.
 *
 * @see src/lib/services/config.ts - getAiProviderIds()
 * @see src-tauri/src/domain/ai.rs - Rust source of truth
 */
export type AiProvider = string;

/** Complete metadata for an AI provider (fetched from Rust backend) */
export interface AiProviderMetadata {
	/** Provider identifier (lowercase string) */
	id: string;
	/** Display name for UI */
	displayName: string;
	/** Whether the provider requires an API key */
	requiresApiKey: boolean;
	/** Default model for this provider */
	defaultModel: string;
	/** Default base URL (if applicable) */
	defaultBaseUrl?: string | null;
}

/** AI provider configuration */
export interface AiProviderConfig {
	provider: AiProvider;
	model: string;
	api_key?: string | null;
	base_url?: string | null;
}

/** A single generated token */
export interface GeneratedToken {
	/** The token content */
	content: string;
	/** Suggested weight (1.0 = normal) */
	suggested_weight: number;
	/** Brief explanation of why this token was suggested */
	rationale?: string | null;
}

/** Status of an AI provider */
export interface AiProviderStatus {
	provider: AiProvider;
	configured: boolean;
	has_api_key: boolean;
	model?: string | null;
	error?: string | null;
}

/** Request to generate tokens for a persona */
export interface TokenGenerationRequest {
	/** The persona name (for context) */
	persona_name: string;
	/** Optional description of the persona */
	persona_description?: string | null;
	/** The granularity level to generate tokens for */
	granularity_name: string;
	/** Number of positive tokens to generate */
	positive_count: number;
	/** Number of negative tokens to generate */
	negative_count: number;
	/** Existing positive tokens to avoid duplicates */
	existing_positive_tokens: string[];
	/** Existing negative tokens to avoid duplicates */
	existing_negative_tokens: string[];
	/** Optional style hints */
	style_hints?: string | null;
	/** The target image generation model ID */
	image_model_id?: string | null;
	/** Custom instructions for AI token generation */
	ai_instructions?: string | null;
	/** The current positive prompt being composed */
	current_positive_prompt?: string | null;
	/** The current negative prompt being composed */
	current_negative_prompt?: string | null;
	/** Token count for the positive prompt */
	positive_token_count?: number | null;
	/** Token count for the negative prompt */
	negative_token_count?: number | null;
	/** Maximum usable tokens */
	max_usable_tokens?: number | null;
}

/** Response from token generation */
export interface TokenGenerationResponse {
	/** Generated positive tokens */
	positive_tokens: GeneratedToken[];
	/** Generated negative tokens */
	negative_tokens: GeneratedToken[];
	/** Provider used */
	provider: AiProvider;
	/** Model used */
	model: string;
}

// ============================================================================
// AI Persona Generation Types
// ============================================================================

/** General physical traits for persona generation */
export interface PhysicalCriteriaGeneral {
	/** Skin tone (e.g., "fair", "medium", "dark") */
	skinTone?: string | null;
	/** Body type (e.g., "slim", "athletic", "curvy") */
	bodyType?: string | null;
	/** Height description (e.g., "short", "average", "tall") */
	height?: string | null;
	/** Apparent age range (e.g., "young adult", "mature") */
	age?: string | null;
	/** Posture type */
	posture?: string | null;
	/** Build proportion */
	buildProportion?: string | null;
	/** Skin texture */
	skinTexture?: string | null;
	/** Distinctive marks (freckles, moles, etc.) */
	distinctiveMarks?: string | null;
	/** Complexion */
	complexion?: string | null;
}

/** Hair characteristics for persona generation */
export interface PhysicalCriteriaHair {
	/** Hair color (main category) */
	color?: string | null;
	/** Hair color shade (specific shade within main color) */
	colorShade?: string | null;
	/** Hair length */
	length?: string | null;
	/** Hair style */
	style?: string | null;
	/** Hair texture */
	texture?: string | null;
}

/** Facial features for persona generation */
export interface PhysicalCriteriaFace {
	/** Eye color */
	eyeColor?: string | null;
	/** Eye shape */
	eyeShape?: string | null;
	/** Face shape */
	faceShape?: string | null;
	/** Nose shape */
	noseShape?: string | null;
	/** Lip shape */
	lipShape?: string | null;
	/** Eyebrow shape */
	eyebrowShape?: string | null;
	/** Chin shape */
	chinShape?: string | null;
	/** Jawline type */
	jawline?: string | null;
	/** Forehead type */
	forehead?: string | null;
	/** Cheekbone prominence */
	cheekbones?: string | null;
	/** Teeth appearance */
	teeth?: string | null;
	/** Smile type */
	smile?: string | null;
}

/** Upper body characteristics for persona generation */
export interface PhysicalCriteriaUpperBody {
	/** Upper body build */
	build?: string | null;
	/** Shoulder type */
	shoulders?: string | null;
	/** Neck type */
	neck?: string | null;
	/** Chest/bust size */
	chest?: string | null;
	/** Arm type */
	arms?: string | null;
	/** Back type */
	back?: string | null;
	/** Hand type */
	hands?: string | null;
	/** Nail type */
	nails?: string | null;
}

/** Midsection characteristics for persona generation */
export interface PhysicalCriteriaMidsection {
	/** Waist type */
	waist?: string | null;
	/** Hip type */
	hips?: string | null;
}

/** Lower body characteristics for persona generation */
export interface PhysicalCriteriaLowerBody {
	/** Leg type */
	legs?: string | null;
	/** Lower body build */
	build?: string | null;
	/** Feet type */
	feet?: string | null;
}

/** Physical criteria organized by body region for AI persona generation */
export interface PhysicalCriteria {
	/** General physical traits */
	general?: PhysicalCriteriaGeneral | null;
	/** Hair characteristics */
	hair?: PhysicalCriteriaHair | null;
	/** Facial features */
	face?: PhysicalCriteriaFace | null;
	/** Upper body characteristics */
	upperBody?: PhysicalCriteriaUpperBody | null;
	/** Midsection characteristics */
	midsection?: PhysicalCriteriaMidsection | null;
	/** Lower body characteristics */
	lowerBody?: PhysicalCriteriaLowerBody | null;
}

/** Request for AI-based persona generation */
export interface AiPersonaGenerationRequest {
	/** Persona name (required) */
	name: string;
	/** Desired visual style (e.g., "realistic", "anime", "manga") */
	style: string;
	/** Character description including age, background, biography */
	characterDescription: string;
	/** Physical criteria organized by body region (optional) */
	physicalCriteria?: PhysicalCriteria;
	/** Custom instructions for the AI (optional) */
	aiInstructions?: string | null;
	/** Target image model for tokenizer awareness (optional) */
	imageModelId?: string;
	/** Existing tags from other personas (for AI to prefer over new ones) */
	existingTags?: string[];
}

/** Generated tokens organized by granularity level */
export interface GeneratedTokensByGranularity {
	/** Style tokens */
	style: GeneratedToken[];
	/** General physical trait tokens */
	general: GeneratedToken[];
	/** Hair-related tokens */
	hair: GeneratedToken[];
	/** Face-related tokens */
	face: GeneratedToken[];
	/** Upper body tokens */
	upper_body: GeneratedToken[];
	/** Midsection tokens */
	midsection: GeneratedToken[];
	/** Lower body tokens */
	lower_body: GeneratedToken[];
}

/** Response from AI persona generation */
export interface AiPersonaGenerationResponse {
	/** Elaborated persona description */
	description: string;
	/** Inferred tags from style and character description */
	tags: string[];
	/** Generated tokens organized by granularity */
	tokens: GeneratedTokensByGranularity;
	/** Provider that handled the request */
	provider: AiProvider;
	/** Model used for generation */
	model: string;
}
