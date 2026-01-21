//! AI provider service
//!
//! Provides a unified interface for AI-powered generation using various providers.
//! Supports `OpenAI`, Anthropic, Google, xAI, and Ollama.

use genai::chat::{ChatMessage, ChatOptions, ChatRequest, ChatResponse, JsonSpec};
use genai::resolver::{AuthData, AuthResolver};
use genai::Client;
use serde_json::json;

use crate::domain::ai::{
    AiPersonaGenerationRequest, AiPersonaGenerationResponse, AiProvider, AiProviderConfig,
    GeneratedToken, GeneratedTokensByGranularity, TokenGenerationRequest, TokenGenerationResponse,
};
use crate::domain::DEFAULT_IMAGE_MODEL_ID;
use crate::error::AppError;
use crate::infrastructure::tokenizer::{
    get_config_for_model, get_prompt_context_for_model, ImageModelPromptContext, TokenizerConfig,
};

// ============================================================================
// Provider Configuration
// ============================================================================

/// Build the model identifier for the genai client
fn build_genai_model_identifier(config: &AiProviderConfig) -> String {
    match config.provider {
        AiProvider::OpenAI => format!("openai/{}", config.model),
        AiProvider::Anthropic => format!("anthropic/{}", config.model),
        AiProvider::Google => format!("gemini/{}", config.model),
        // xAI: genai auto-detects models starting with "grok" (no prefix needed)
        AiProvider::XAi => config.model.clone(),
        // Ollama: genai auto-detects based on model name (no prefix needed)
        AiProvider::Ollama => config.model.clone(),
    }
}

// ============================================================================
// Persona Generation
// ============================================================================
//
// Creates complete persona profiles with tokens organized by body region.

/// Build the system prompt for AI persona generation
fn build_persona_generation_system_prompt(
    prompt_context: &ImageModelPromptContext,
    tokenizer_config: &TokenizerConfig,
    existing_tags: &[String],
) -> String {
    let existing_tags_section = if existing_tags.is_empty() {
        String::new()
    } else {
        format!(
            "\nEXISTING TAGS (prefer these over creating new similar ones):\n{}",
            existing_tags.join(", ")
        )
    };

    format!(
        r#"You are an expert character designer and prompt engineer for {model_name} ({family} family) image generation.

Your task is to create a complete persona profile with descriptive tokens organized by body region.
Maximum token budget: {total_tokens} tokens.

TOKEN GENERATION RULES:
1. Generate visually descriptive tokens suitable for AI image generation
2. Each token should be specific and concrete (e.g., "auburn wavy hair" not just "hair")
3. Tokens should be POSITIVE descriptions (what to include, not what to exclude)
4. DO NOT generate clothing, accessories, or outfit tokens unless explicitly mentioned
5. Focus on physical characteristics and style only

GRANULARITY ORGANIZATION:
- style: Style tokens (e.g., "masterpiece", "photorealistic", "anime style", "oil painting", "cinematic")
- general: Overall physical traits (skin tone, body type, age, ethnicity features)
- hair: Hair color, length, style, texture
- face: Eyes, face shape, facial features
- upper_body: Shoulders, arms, chest, back (physical build only)
- midsection: Waist, hips, midriff (physical traits only)
- lower_body: Legs, thighs (physical traits only)

TAG INFERENCE:
Derive 1-3 relevant tags from the style and description (e.g., "fantasy", "female", "anime").{existing_tags_section}

DESCRIPTION ELABORATION:
Expand the user's character description into a cohesive narrative suitable for consistent image generation."#,
        model_name = prompt_context.display_name,
        family = prompt_context.family,
        total_tokens = tokenizer_config.usable_tokens,
        existing_tags_section = existing_tags_section,
    )
}

/// Build the user prompt for AI persona generation
fn build_persona_generation_user_prompt(request: &AiPersonaGenerationRequest) -> String {
    let mut sections = Vec::new();

    // Basic information
    sections.push(format!("CHARACTER NAME: {}", request.name));
    sections.push(format!("DESIRED STYLE: {}", request.style));
    if request.character_description.is_empty() {
        sections.push(
            "CHARACTER DESCRIPTION: Not provided - derive from style and physical criteria"
                .to_string(),
        );
    } else {
        sections.push(format!(
            "CHARACTER DESCRIPTION:\n```\n{}\n```",
            request.character_description
        ));
    }

    // Physical criteria by granularity - JSON format with subgroups matching the UI form
    // This allows AI to know which characteristics it has freedom to define
    let no_pref = "No preference";
    let criteria = &request.physical_criteria;

    let general = criteria.general.as_ref();
    let hair = criteria.hair.as_ref();
    let face = criteria.face.as_ref();
    let upper = criteria.upper_body.as_ref();
    let mid = criteria.midsection.as_ref();
    let lower = criteria.lower_body.as_ref();

    let physical_json = json!({
        "general": {
            "sex": general.and_then(|g| g.sex.as_deref()).unwrap_or(no_pref),
            "age": general.and_then(|g| g.age.as_deref()).unwrap_or(no_pref),
            "skin": {
                "tone": general.and_then(|g| g.skin_tone.as_deref()).unwrap_or(no_pref),
                "complexion": general.and_then(|g| g.complexion.as_deref()).unwrap_or(no_pref),
                "texture": general.and_then(|g| g.skin_texture.as_deref()).unwrap_or(no_pref),
                "distinctiveMarks": general.and_then(|g| g.distinctive_marks.as_deref()).unwrap_or(no_pref)
            },
            "body": {
                "type": general.and_then(|g| g.body_type.as_deref()).unwrap_or(no_pref),
                "height": general.and_then(|g| g.height.as_deref()).unwrap_or(no_pref),
                "proportions": general.and_then(|g| g.proportions.as_deref()).unwrap_or(no_pref),
                "posture": general.and_then(|g| g.posture.as_deref()).unwrap_or(no_pref)
            }
        },
        "hair": {
            "color": {
                "main": hair.and_then(|h| h.color.as_deref()).unwrap_or(no_pref),
                "shade": hair.and_then(|h| h.color_shade.as_deref()).unwrap_or(no_pref)
            },
            "attributes": {
                "length": hair.and_then(|h| h.length.as_deref()).unwrap_or(no_pref),
                "style": hair.and_then(|h| h.style.as_deref()).unwrap_or(no_pref),
                "texture": hair.and_then(|h| h.texture.as_deref()).unwrap_or(no_pref)
            }
        },
        "face": {
            "structure": {
                "forehead": face.and_then(|f| f.forehead.as_deref()).unwrap_or(no_pref),
                "shape": face.and_then(|f| f.face_shape.as_deref()).unwrap_or(no_pref),
                "cheekbones": face.and_then(|f| f.cheekbones.as_deref()).unwrap_or(no_pref),
                "jawline": face.and_then(|f| f.jawline.as_deref()).unwrap_or(no_pref),
                "chin": face.and_then(|f| f.chin_shape.as_deref()).unwrap_or(no_pref)
            },
            "eyebrows": {
                "shape": face.and_then(|f| f.eyebrow_shape.as_deref()).unwrap_or(no_pref)
            },
            "eyes": {
                "color": face.and_then(|f| f.eye_color.as_deref()).unwrap_or(no_pref),
                "shape": face.and_then(|f| f.eye_shape.as_deref()).unwrap_or(no_pref)
            },
            "nose": {
                "shape": face.and_then(|f| f.nose_shape.as_deref()).unwrap_or(no_pref)
            },
            "mouth": {
                "lipShape": face.and_then(|f| f.lip_shape.as_deref()).unwrap_or(no_pref),
                "teeth": face.and_then(|f| f.teeth.as_deref()).unwrap_or(no_pref)
            }
        },
        "upperBody": {
            "neck": upper.and_then(|u| u.neck.as_deref()).unwrap_or(no_pref),
            "torso": {
                "build": upper.and_then(|u| u.build.as_deref()).unwrap_or(no_pref),
                "shoulders": upper.and_then(|u| u.shoulders.as_deref()).unwrap_or(no_pref),
                "back": upper.and_then(|u| u.back.as_deref()).unwrap_or(no_pref),
                "chest": upper.and_then(|u| u.chest.as_deref()).unwrap_or(no_pref)
            },
            "upperLimbs": {
                "arms": upper.and_then(|u| u.arms.as_deref()).unwrap_or(no_pref),
                "hands": upper.and_then(|u| u.hands.as_deref()).unwrap_or(no_pref),
                "nails": upper.and_then(|u| u.nails.as_deref()).unwrap_or(no_pref)
            }
        },
        "midsection": {
            "waist": mid.and_then(|m| m.waist.as_deref()).unwrap_or(no_pref),
            "hips": mid.and_then(|m| m.hips.as_deref()).unwrap_or(no_pref)
        },
        "lowerBody": {
            "legs": lower.and_then(|l| l.legs.as_deref()).unwrap_or(no_pref),
            "build": lower.and_then(|l| l.build.as_deref()).unwrap_or(no_pref),
            "feet": lower.and_then(|l| l.feet.as_deref()).unwrap_or(no_pref)
        }
    });

    sections.push(format!(
        "PHYSICAL SPECIFICATIONS:\n```json\n{}\n```",
        serde_json::to_string_pretty(&physical_json).unwrap()
    ));

    // Custom instructions
    if let Some(instructions) = &request.ai_instructions {
        if !instructions.is_empty() {
            sections.push(format!("CUSTOM INSTRUCTIONS:\n```\n{instructions}\n```"));
        }
    }

    // Constraints
    sections.push(
        r"CONSTRAINTS:
- For characteristics marked 'No preference', you have creative freedom to define appropriate values that fit the persona's style and description
- If physical criteria contradict the description, the description takes precedence
- Do NOT generate clothing or accessory tokens unless explicitly described
- Each granularity should have relevant tokens
- Use the specified style consistently across all tokens
- Ensure tokens are suitable for image generation prompts"
            .to_string(),
    );

    // Section: Expected Output Format
    let output_section = r#"EXPECTED OUTPUT:
Respond with a JSON object containing:
- "description" (string): Elaborated persona description as a cohesive narrative
- "tags" (array of strings): 1-3 relevant tags inferred from style and description
- "tokens" (object): Token arrays organized by body region

Each token object contains:
- "content" (string, required): The token text
- "suggested_weight" (number, required): Weight value where 1.0 is normal emphasis
- "rationale" (string, optional): Brief explanation for this token

Example format:
```json
{
  "description": "A graceful elven warrior with silver hair...",
  "tags": ["fantasy", "female", "elf"],
  "tokens": {
    "style": [
      {"content": "masterpiece", "suggested_weight": 1.2, "rationale": "Quality boost"}
    ],
    "general": [
      {"content": "fair skin", "suggested_weight": 1.0, "rationale": "Elven complexion"}
    ],
    "hair": [
      {"content": "long silver hair", "suggested_weight": 1.1, "rationale": "Distinctive feature"}
    ],
    "face": [
      {"content": "pointed ears", "suggested_weight": 1.2, "rationale": "Elven trait"}
    ],
    "upper_body": [
      {"content": "slender build", "suggested_weight": 1.0, "rationale": "Elven physique"}
    ],
    "midsection": [
      {"content": "narrow waist", "suggested_weight": 1.0, "rationale": "Athletic build"}
    ],
    "lower_body": [
      {"content": "long legs", "suggested_weight": 1.0, "rationale": "Tall stature"}
    ]
  }
}
```"#;

    sections.push(output_section.to_string());

    sections.join("\n\n")
}

/// Build the JSON schema for AI persona generation response
fn build_persona_generation_json_schema() -> serde_json::Value {
    let token_array_schema = json!({
        "type": "array",
        "items": {
            "type": "object",
            "properties": {
                "content": { "type": "string" },
                "suggested_weight": { "type": "number" },
                "rationale": { "type": "string" }
            },
            "required": ["content", "suggested_weight"]
        }
    });

    json!({
        "type": "object",
        "properties": {
            "description": {
                "type": "string",
                "description": "Elaborated persona description"
            },
            "tags": {
                "type": "array",
                "items": { "type": "string" },
                "maxItems": 3,
                "description": "1-3 relevant tags inferred from style and description"
            },
            "tokens": {
                "type": "object",
                "properties": {
                    "style": token_array_schema,
                    "general": token_array_schema,
                    "hair": token_array_schema,
                    "face": token_array_schema,
                    "upper_body": token_array_schema,
                    "midsection": token_array_schema,
                    "lower_body": token_array_schema
                },
                "required": ["style", "general", "hair", "face", "upper_body", "midsection", "lower_body"]
            }
        },
        "required": ["description", "tags", "tokens"]
    })
}

/// Internal structure for parsing AI persona generation response
#[derive(Debug, Clone, serde::Deserialize)]
struct PersonaGenerationRaw {
    description: String,
    tags: Vec<String>,
    tokens: GeneratedTokensByGranularity,
}

/// Parse the AI response for persona generation
fn parse_persona_response(content: &str) -> Result<PersonaGenerationRaw, AppError> {
    // Try to extract JSON object from the response
    let json_str = if let Some(start) = content.find('{') {
        if let Some(end) = content.rfind('}') {
            &content[start..=end]
        } else {
            content
        }
    } else {
        content
    };

    serde_json::from_str(json_str).map_err(|e| {
        AppError::Internal(format!(
            "Failed to parse AI persona response: {e}. Response was: {content}"
        ))
    })
}

/// Generate a complete persona using AI
///
/// Takes user inputs (name, style, character description, physical criteria) and
/// generates a fully-formed persona with tokens organized by granularity.
pub async fn generate_persona(
    config: &AiProviderConfig,
    request: &AiPersonaGenerationRequest,
) -> Result<AiPersonaGenerationResponse, AppError> {
    // Build client with API key from config
    let client = if let Some(api_key) = &config.api_key {
        let api_key = api_key.clone();
        let auth_resolver = AuthResolver::from_resolver_fn(
            move |_model_iden| -> Result<Option<AuthData>, genai::resolver::Error> {
                Ok(Some(AuthData::from_single(api_key.clone())))
            },
        );
        Client::builder().with_auth_resolver(auth_resolver).build()
    } else {
        // Fall back to environment variables (for Ollama or if no key provided)
        Client::default()
    };

    // Get model context for the selected image model
    let image_model_id_str = request.image_model_id.as_deref();
    let prompt_context = get_prompt_context_for_model(image_model_id_str);
    let tokenizer_config =
        get_config_for_model(image_model_id_str.unwrap_or(DEFAULT_IMAGE_MODEL_ID));

    let system_prompt = build_persona_generation_system_prompt(
        &prompt_context,
        &tokenizer_config,
        &request.existing_tags,
    );
    let user_prompt = build_persona_generation_user_prompt(request);

    let chat_request = ChatRequest::default()
        .with_system(system_prompt)
        .append_message(ChatMessage::user(user_prompt));

    // Create ChatOptions with structured response format for API-level schema enforcement
    let json_schema = build_persona_generation_json_schema();
    let chat_options =
        ChatOptions::default().with_response_format(JsonSpec::new("persona", json_schema));

    let model_id = build_genai_model_identifier(config);

    let response: ChatResponse = client
        .exec_chat(&model_id, chat_request, Some(&chat_options))
        .await
        .map_err(|e| AppError::Internal(format!("AI persona generation failed: {e}")))?;

    let content = response
        .first_text()
        .ok_or_else(|| AppError::Internal("No response content from AI".to_string()))?;

    let parsed = parse_persona_response(content)?;

    Ok(AiPersonaGenerationResponse {
        description: parsed.description,
        tags: parsed.tags,
        tokens: parsed.tokens,
        provider: config.provider,
        model: config.model.clone(),
    })
}

// ============================================================================
// Token Generation
// ============================================================================
//
// Generates additional positive/negative tokens during prompt composition.

/// Build the system prompt for token generation
fn build_token_generation_system_prompt(
    prompt_context: &ImageModelPromptContext,
    tokenizer_config: &crate::infrastructure::tokenizer::TokenizerConfig,
) -> String {
    format!(
        r"You are an expert prompt engineer for {model_name} ({family} family) image generation.

Generate visually descriptive tokens for AI image prompts. Token budget: {limit} tokens.

TOKEN REQUIREMENTS:
- Visually specific and descriptive
- Positive: desirable visual characteristics
- Negative: elements to exclude",
        model_name = prompt_context.display_name,
        family = prompt_context.family,
        limit = tokenizer_config.usable_tokens,
    )
}

/// Build the user prompt for token generation
fn build_token_generation_user_prompt(request: &TokenGenerationRequest) -> String {
    let model_id = request.image_model_id.as_deref();
    let tokenizer_config = get_config_for_model(model_id.unwrap_or(DEFAULT_IMAGE_MODEL_ID));
    let mut sections = Vec::new();

    // Section 1: Persona Information
    let mut persona_section = format!("PERSONA: {}", request.persona_name);
    if let Some(desc) = &request.persona_description {
        if !desc.is_empty() {
            persona_section.push_str(&format!("\nDescription:\n```\n{desc}\n```"));
        }
    }
    sections.push(persona_section);

    // Section 2: Current Prompt State
    if request.current_positive_prompt.is_some() || request.current_negative_prompt.is_some() {
        let mut state_section = String::from("CURRENT PROMPTS:");
        let max_tokens = request
            .max_usable_tokens
            .unwrap_or(tokenizer_config.usable_tokens);

        if let Some(pos) = &request.current_positive_prompt {
            if !pos.is_empty() {
                let pos_words = pos.split_whitespace().count();
                let pos_count = request.positive_token_count.unwrap_or(0);
                let pos_remaining = max_tokens.saturating_sub(pos_count);
                state_section.push_str(&format!(
                    "\nPositive ({pos_words} words; {pos_count}/{max_tokens} tokens, {pos_remaining} remaining): {pos}"
                ));
            }
        }

        if let Some(neg) = &request.current_negative_prompt {
            if !neg.is_empty() {
                let neg_words = neg.split_whitespace().count();
                let neg_count = request.negative_token_count.unwrap_or(0);
                let neg_remaining = max_tokens.saturating_sub(neg_count);
                state_section.push_str(&format!(
                    "\nNegative ({neg_words} words; {neg_count}/{max_tokens} tokens, {neg_remaining} remaining): {neg}"
                ));
            }
        }

        sections.push(state_section);
    }

    // Section 3: Task Specification
    sections.push(
        "TASK: Generate positive and negative tokens based on the context below.".to_string(),
    );

    // Section 4: Context/Action
    if let Some(hints) = &request.style_hints {
        if !hints.is_empty() {
            sections.push(format!("CONTEXT/ACTION:\n```\n{hints}\n```"));
        }
    }

    // Section 5: Custom AI Instructions
    if let Some(instructions) = &request.ai_instructions {
        if !instructions.is_empty() {
            sections.push(format!("CUSTOM INSTRUCTIONS:\n{instructions}"));
        }
    }

    // Section 6: Constraints
    let max_tokens = request
        .max_usable_tokens
        .unwrap_or(tokenizer_config.usable_tokens);
    let mut constraints = vec![
        "Generate tokens based ONLY on the provided persona and context. Do not invent characteristics not mentioned.".to_string(),
        "Do not repeat tokens already in the current prompts".to_string(),
    ];

    // Positive token constraints
    if !request.existing_positive_tokens.is_empty() {
        constraints.push(format!(
            "Avoid these existing positive tokens: {}",
            request.existing_positive_tokens.join(", ")
        ));
    }

    // Negative token constraints
    if !request.existing_negative_tokens.is_empty() {
        constraints.push(format!(
            "Avoid these existing negative tokens: {}",
            request.existing_negative_tokens.join(", ")
        ));
    }

    // Token budget warnings
    let pos_count = request.positive_token_count.unwrap_or(0);
    if pos_count > max_tokens / 2 {
        let remaining = max_tokens.saturating_sub(pos_count);
        constraints.push(format!(
            "Positive prompt budget is limited ({remaining} remaining) - prioritize high-impact tokens"
        ));
    }

    let neg_count = request.negative_token_count.unwrap_or(0);
    if neg_count > max_tokens / 2 {
        let remaining = max_tokens.saturating_sub(neg_count);
        constraints.push(format!(
            "Negative prompt budget is limited ({remaining} remaining) - prioritize high-impact tokens"
        ));
    }

    sections.push(format!("CONSTRAINTS:\n- {}", constraints.join("\n- ")));

    // Section 7: Expected Output Format
    let output_section = r#"EXPECTED OUTPUT:
Respond with a JSON object containing two arrays: "positive" and "negative".
Each array contains token objects with:
- "content" (string, required): The token text
- "suggested_weight" (number, required): Weight value where 1.0 is normal emphasis
- "rationale" (string, optional): Brief explanation for this token

Example format:
```json
{
  "positive": [
    {"content": "detailed eyes", "suggested_weight": 1.2, "rationale": "Enhances facial detail"}
  ],
  "negative": [
    {"content": "blurry", "suggested_weight": 1.0, "rationale": "Prevents low quality output"}
  ]
}
```"#;

    sections.push(output_section.to_string());

    sections.join("\n\n")
}

/// Internal structure for parsing AI response
#[derive(Debug, Clone, serde::Deserialize)]
struct TokensRaw {
    positive: Vec<GeneratedToken>,
    negative: Vec<GeneratedToken>,
}

/// Parse the AI response into positive and negative tokens
fn parse_token_generation_response(
    content: &str,
) -> Result<(Vec<GeneratedToken>, Vec<GeneratedToken>), AppError> {
    // Try to extract JSON object from the response
    let json_str = if let Some(start) = content.find('{') {
        if let Some(end) = content.rfind('}') {
            &content[start..=end]
        } else {
            content
        }
    } else {
        content
    };

    let parsed: TokensRaw = serde_json::from_str(json_str).map_err(|e| {
        AppError::Internal(format!(
            "Failed to parse AI response: {e}. Response was: {content}"
        ))
    })?;

    Ok((parsed.positive, parsed.negative))
}

/// Build the JSON schema for token generation response
fn build_token_generation_json_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "positive": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "content": { "type": "string" },
                        "suggested_weight": { "type": "number" },
                        "rationale": { "type": "string" }
                    },
                    "required": ["content", "suggested_weight"]
                }
            },
            "negative": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "content": { "type": "string" },
                        "suggested_weight": { "type": "number" },
                        "rationale": { "type": "string" }
                    },
                    "required": ["content", "suggested_weight"]
                }
            }
        },
        "required": ["positive", "negative"]
    })
}

/// Generate tokens using an AI provider
pub async fn generate_tokens(
    config: &AiProviderConfig,
    request: &TokenGenerationRequest,
) -> Result<TokenGenerationResponse, AppError> {
    // Build client with API key from config (not environment variable)
    let client = if let Some(api_key) = &config.api_key {
        let api_key = api_key.clone();
        let auth_resolver = AuthResolver::from_resolver_fn(
            move |_model_iden| -> Result<Option<AuthData>, genai::resolver::Error> {
                Ok(Some(AuthData::from_single(api_key.clone())))
            },
        );
        Client::builder().with_auth_resolver(auth_resolver).build()
    } else {
        // Fall back to environment variables (for Ollama or if no key provided)
        Client::default()
    };

    let model_id_str = request.image_model_id.as_deref();
    let prompt_context = get_prompt_context_for_model(model_id_str);
    let tokenizer_config = get_config_for_model(model_id_str.unwrap_or(DEFAULT_IMAGE_MODEL_ID));

    let system_prompt = build_token_generation_system_prompt(&prompt_context, &tokenizer_config);
    let user_prompt = build_token_generation_user_prompt(request);

    let chat_request = ChatRequest::default()
        .with_system(system_prompt)
        .append_message(ChatMessage::user(user_prompt));

    // Create ChatOptions with structured response format for API-level schema enforcement
    let json_schema = build_token_generation_json_schema();
    let chat_options =
        ChatOptions::default().with_response_format(JsonSpec::new("tokens", json_schema));

    let model_id = build_genai_model_identifier(config);

    let response: ChatResponse = client
        .exec_chat(&model_id, chat_request, Some(&chat_options))
        .await
        .map_err(|e| AppError::Internal(format!("AI request failed: {e}")))?;

    let content = response
        .first_text()
        .ok_or_else(|| AppError::Internal("No response content from AI".to_string()))?;

    let (positive_tokens, negative_tokens) = parse_token_generation_response(content)?;

    Ok(TokenGenerationResponse {
        positive_tokens,
        negative_tokens,
        provider: config.provider,
        model: config.model.clone(),
    })
}
