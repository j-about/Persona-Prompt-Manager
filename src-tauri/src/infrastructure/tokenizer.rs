//! Model-agnostic tokenizer service
//!
//! Provides token counting functionality for various image generation models.
//! Supports dynamic tokenizer loading from `HuggingFace` based on the model being used.

use std::collections::HashMap;
use std::sync::RwLock;
use tokenizers::Tokenizer;

use crate::domain::DEFAULT_IMAGE_MODEL_ID;
use crate::error::AppError;

/// Default tokenizer for unknown models (CLIP for Stable Diffusion compatibility)
const DEFAULT_TOKENIZER_ID: &str = "openai/clip-vit-large-patch14";
const DEFAULT_MAX_TOKENS: usize = 77;
const DEFAULT_USABLE_TOKENS: usize = 75;

/// Tokenizer configuration for a specific model
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenizerConfig {
    /// The `HuggingFace` tokenizer ID to use
    pub tokenizer_id: String,
    /// Maximum tokens allowed by the model
    pub max_tokens: usize,
    /// Usable tokens after accounting for special tokens
    pub usable_tokens: usize,
}

impl Default for TokenizerConfig {
    fn default() -> Self {
        Self {
            tokenizer_id: DEFAULT_TOKENIZER_ID.to_string(),
            max_tokens: DEFAULT_MAX_TOKENS,
            usable_tokens: DEFAULT_USABLE_TOKENS,
        }
    }
}

/// Known model → tokenizer mappings (base models only)
fn get_known_mappings() -> HashMap<&'static str, TokenizerConfig> {
    let mut mappings = HashMap::new();

    // =========================================================================
    // D - DeepFloyd IF (Stability AI)
    // =========================================================================

    mappings.insert(
        "DeepFloyd/IF-I-XL-v1.0",
        TokenizerConfig {
            tokenizer_id: "google/t5-v1_1-xxl".to_string(),
            max_tokens: 77,
            usable_tokens: 75,
        },
    );

    // =========================================================================
    // H - Hunyuan (Tencent)
    // =========================================================================

    mappings.insert(
        "Tencent-Hunyuan/HunyuanDiT-v1.2",
        TokenizerConfig {
            tokenizer_id: "google/t5-v1_1-xxl".to_string(),
            max_tokens: 256,
            usable_tokens: 250,
        },
    );

    mappings.insert(
        "tencent/HunyuanImage-3.0",
        TokenizerConfig {
            tokenizer_id: "google/t5-v1_1-xxl".to_string(),
            max_tokens: 256,
            usable_tokens: 250,
        },
    );

    // =========================================================================
    // K - Kandinsky (Sber AI), Kolors (Kwai)
    // =========================================================================

    mappings.insert(
        "kandinsky-community/kandinsky-2-2-decoder",
        TokenizerConfig {
            tokenizer_id: "openai/clip-vit-large-patch14".to_string(),
            max_tokens: 77,
            usable_tokens: 75,
        },
    );

    mappings.insert(
        "ai-forever/kandinsky-3.1",
        TokenizerConfig {
            tokenizer_id: "openai/clip-vit-large-patch14".to_string(),
            max_tokens: 77,
            usable_tokens: 75,
        },
    );

    mappings.insert(
        "Kwai-Kolors/Kolors",
        TokenizerConfig {
            tokenizer_id: "google/t5-v1_1-xxl".to_string(),
            max_tokens: 256,
            usable_tokens: 250,
        },
    );

    // =========================================================================
    // P - PixArt (PixArt-alpha)
    // =========================================================================

    mappings.insert(
        "PixArt-alpha/PixArt-XL-2-1024-MS",
        TokenizerConfig {
            tokenizer_id: "google/t5-v1_1-xxl".to_string(),
            max_tokens: 256,
            usable_tokens: 250,
        },
    );

    // =========================================================================
    // S - Stable Diffusion, Stable Cascade (Stability AI)
    // =========================================================================

    mappings.insert(
        "stabilityai/stable-cascade",
        TokenizerConfig {
            tokenizer_id: "openai/clip-vit-large-patch14".to_string(),
            max_tokens: 77,
            usable_tokens: 75,
        },
    );

    mappings.insert(
        "CompVis/stable-diffusion-v1-4",
        TokenizerConfig {
            tokenizer_id: "openai/clip-vit-large-patch14".to_string(),
            max_tokens: 77,
            usable_tokens: 75,
        },
    );

    mappings.insert(
        "runwayml/stable-diffusion-v1-5",
        TokenizerConfig {
            tokenizer_id: "openai/clip-vit-large-patch14".to_string(),
            max_tokens: 77,
            usable_tokens: 75,
        },
    );

    mappings.insert(
        "stable-diffusion-v1-5/stable-diffusion-v1-5",
        TokenizerConfig {
            tokenizer_id: "openai/clip-vit-large-patch14".to_string(),
            max_tokens: 77,
            usable_tokens: 75,
        },
    );

    mappings.insert(
        "stabilityai/stable-diffusion-2",
        TokenizerConfig {
            tokenizer_id: "laion/CLIP-ViT-H-14-laion2B-s32B-b79K".to_string(),
            max_tokens: 77,
            usable_tokens: 75,
        },
    );

    mappings.insert(
        "stabilityai/stable-diffusion-2-1",
        TokenizerConfig {
            tokenizer_id: "laion/CLIP-ViT-H-14-laion2B-s32B-b79K".to_string(),
            max_tokens: 77,
            usable_tokens: 75,
        },
    );

    // Stable Diffusion XL
    mappings.insert(
        "stabilityai/stable-diffusion-xl-base-1.0",
        TokenizerConfig {
            tokenizer_id: "openai/clip-vit-large-patch14".to_string(),
            max_tokens: 77,
            usable_tokens: 75,
        },
    );

    // SDXL Turbo (distilled SDXL)
    mappings.insert(
        "stabilityai/sdxl-turbo",
        TokenizerConfig {
            tokenizer_id: "openai/clip-vit-large-patch14".to_string(),
            max_tokens: 77,
            usable_tokens: 75,
        },
    );

    // =========================================================================
    // W
    // =========================================================================

    // Würstchen (efficient latent diffusion)
    mappings.insert(
        "warp-ai/wuerstchen",
        TokenizerConfig {
            tokenizer_id: "openai/clip-vit-large-patch14".to_string(),
            max_tokens: 77,
            usable_tokens: 75,
        },
    );

    mappings
}

/// Global tokenizer cache (`model_id` → Tokenizer)
static TOKENIZER_CACHE: RwLock<Option<HashMap<String, Tokenizer>>> = RwLock::new(None);

/// Get or load a tokenizer for the specified tokenizer ID
fn get_or_load_tokenizer(tokenizer_id: &str) -> Result<Tokenizer, AppError> {
    // Check if already cached
    {
        let cache = TOKENIZER_CACHE.read().map_err(|_| {
            AppError::Internal("Failed to acquire tokenizer cache read lock".to_string())
        })?;

        if let Some(ref cache_map) = *cache {
            if let Some(tokenizer) = cache_map.get(tokenizer_id) {
                return Ok(tokenizer.clone());
            }
        }
    }

    // Load the tokenizer
    let tokenizer = Tokenizer::from_pretrained(tokenizer_id, None).map_err(|e| {
        AppError::Internal(format!("Failed to load tokenizer '{tokenizer_id}': {e}"))
    })?;

    // Cache it
    {
        let mut cache = TOKENIZER_CACHE.write().map_err(|_| {
            AppError::Internal("Failed to acquire tokenizer cache write lock".to_string())
        })?;

        if cache.is_none() {
            *cache = Some(HashMap::new());
        }

        if let Some(ref mut cache_map) = *cache {
            cache_map.insert(tokenizer_id.to_string(), tokenizer.clone());
        }
    }

    Ok(tokenizer)
}

/// Get the tokenizer configuration for a model
#[must_use]
pub fn get_config_for_model(model_id: &str) -> TokenizerConfig {
    let mappings = get_known_mappings();

    // Try exact match first
    if let Some(config) = mappings.get(model_id) {
        return config.clone();
    }

    // Try to match by prefix/family
    let model_lower = model_id.to_lowercase();

    // =========================================================================
    // T5-based models (256 tokens)
    // =========================================================================

    // PixArt models
    if model_lower.contains("pixart") {
        return TokenizerConfig {
            tokenizer_id: "google/t5-v1_1-xxl".to_string(),
            max_tokens: 256,
            usable_tokens: 250,
        };
    }

    // Hunyuan models (Tencent)
    if model_lower.contains("hunyuan") {
        return TokenizerConfig {
            tokenizer_id: "google/t5-v1_1-xxl".to_string(),
            max_tokens: 256,
            usable_tokens: 250,
        };
    }

    // Kolors (Kwai)
    if model_lower.contains("kolors") {
        return TokenizerConfig {
            tokenizer_id: "google/t5-v1_1-xxl".to_string(),
            max_tokens: 256,
            usable_tokens: 250,
        };
    }

    // DeepFloyd IF (T5 encoder but 77 token limit)
    if model_lower.contains("deepfloyd") || model_lower.contains("if-i-") {
        return TokenizerConfig {
            tokenizer_id: "google/t5-v1_1-xxl".to_string(),
            max_tokens: 77,
            usable_tokens: 75,
        };
    }

    // =========================================================================
    // CLIP-based models (77 tokens)
    // =========================================================================

    // SDXL
    if model_lower.contains("sdxl") || model_lower.contains("stable-diffusion-xl") {
        return TokenizerConfig {
            tokenizer_id: "openai/clip-vit-large-patch14".to_string(),
            max_tokens: 77,
            usable_tokens: 75,
        };
    }

    // Stable Diffusion 2.x (OpenCLIP)
    if model_lower.contains("stable-diffusion-2") {
        return TokenizerConfig {
            tokenizer_id: "laion/CLIP-ViT-H-14-laion2B-s32B-b79K".to_string(),
            max_tokens: 77,
            usable_tokens: 75,
        };
    }

    // Stable Cascade / Würstchen
    if model_lower.contains("cascade") || model_lower.contains("wuerstchen") {
        return TokenizerConfig {
            tokenizer_id: "openai/clip-vit-large-patch14".to_string(),
            max_tokens: 77,
            usable_tokens: 75,
        };
    }

    // Kandinsky models
    if model_lower.contains("kandinsky") {
        return TokenizerConfig {
            tokenizer_id: "openai/clip-vit-large-patch14".to_string(),
            max_tokens: 77,
            usable_tokens: 75,
        };
    }

    // Default to CLIP tokenizer (SD 1.x compatible)
    TokenizerConfig::default()
}

/// Token count result with detailed breakdown
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenCount {
    /// Number of tokens in the text
    pub count: usize,
    /// Maximum allowed tokens for the model
    pub max_tokens: usize,
    /// Usable tokens (excluding special tokens)
    pub usable_tokens: usize,
    /// Whether the text exceeds the token limit
    pub exceeds_limit: bool,
    /// Percentage of limit used (0-100+)
    pub usage_percent: f64,
    /// The model this count is for
    pub model_id: String,
    /// The tokenizer used
    pub tokenizer_id: String,
}

impl TokenCount {
    fn new(count: usize, config: &TokenizerConfig, model_id: &str) -> Self {
        let exceeds_limit = count > config.usable_tokens;
        let usage_percent = if config.usable_tokens > 0 {
            (count as f64 / config.usable_tokens as f64) * 100.0
        } else {
            0.0
        };

        Self {
            count,
            max_tokens: config.max_tokens,
            usable_tokens: config.usable_tokens,
            exceeds_limit,
            usage_percent,
            model_id: model_id.to_string(),
            tokenizer_id: config.tokenizer_id.clone(),
        }
    }
}

/// Count tokens in a text string for a specific model
///
/// Falls back to simple word counting if the tokenizer is not available.
#[must_use]
pub fn count_tokens(text: &str, model_id: Option<&str>) -> TokenCount {
    let model = model_id.unwrap_or(DEFAULT_IMAGE_MODEL_ID);
    let config = get_config_for_model(model);

    let text = text.trim();
    if text.is_empty() {
        return TokenCount::new(0, &config, model);
    }

    // Try to use the real tokenizer
    match get_or_load_tokenizer(&config.tokenizer_id) {
        Ok(tokenizer) => match tokenizer.encode(text, false) {
            Ok(encoding) => TokenCount::new(encoding.get_ids().len(), &config, model),
            Err(_) => simple_token_count(text, &config, model),
        },
        Err(_) => simple_token_count(text, &config, model),
    }
}

/// Simple token counting fallback (word-based approximation)
fn simple_token_count(text: &str, config: &TokenizerConfig, model_id: &str) -> TokenCount {
    let mut count = 0;

    for word in text.split(|c: char| c.is_whitespace() || c == ',') {
        let word = word.trim();
        if word.is_empty() {
            continue;
        }

        count += 1;

        // Add extra for special characters that typically become separate tokens
        count += word
            .chars()
            .filter(|c| matches!(c, '(' | ')' | '[' | ']' | ':' | '|'))
            .count();
    }

    TokenCount::new(count, config, model_id)
}

/// Count tokens in multiple text strings
#[must_use]
pub fn count_tokens_batch(texts: &[&str], model_id: Option<&str>) -> Vec<TokenCount> {
    texts
        .iter()
        .map(|text| count_tokens(text, model_id))
        .collect()
}

/// Get information about the tokenizer for a model
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenizerInfo {
    pub model_id: String,
    pub tokenizer_id: String,
    pub available: bool,
    pub max_tokens: usize,
    pub usable_tokens: usize,
}

#[must_use]
pub fn get_tokenizer_info(model_id: Option<&str>) -> TokenizerInfo {
    let model = model_id.unwrap_or(DEFAULT_IMAGE_MODEL_ID);
    let config = get_config_for_model(model);
    let available = get_or_load_tokenizer(&config.tokenizer_id).is_ok();

    TokenizerInfo {
        model_id: model.to_string(),
        tokenizer_id: config.tokenizer_id,
        available,
        max_tokens: config.max_tokens,
        usable_tokens: config.usable_tokens,
    }
}

/// Get list of all known model mappings
#[must_use]
pub fn get_known_models() -> Vec<TokenizerInfo> {
    let mut models: Vec<TokenizerInfo> = get_known_mappings()
        .iter()
        .map(|(model_id, config)| TokenizerInfo {
            model_id: (*model_id).to_string(),
            tokenizer_id: config.tokenizer_id.clone(),
            available: true, // Will be checked lazily
            max_tokens: config.max_tokens,
            usable_tokens: config.usable_tokens,
        })
        .collect();

    models.sort_by(|a, b| {
        let name_a = a.model_id.rsplit('/').next().unwrap_or(&a.model_id);
        let name_b = b.model_id.rsplit('/').next().unwrap_or(&b.model_id);
        name_a.cmp(name_b)
    });
    models
}

// ============================================================================
// Prompt Engineering Context (for AI token generation)
// ============================================================================

/// Prompt engineering context for an image generation model
///
/// Contains model-specific information needed to generate appropriate
/// tokens for image generation prompts.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageModelPromptContext {
    /// Human-readable display name (e.g., "Stable Diffusion XL")
    pub display_name: String,
    /// Model family identifier (sdxl, pixart, sd2, sd15, kandinsky)
    pub family: String,
}

/// Get prompt engineering context for an image generation model
///
/// This is the single source of truth for model-specific prompt engineering knowledge.
/// Uses the same model detection logic as `get_config_for_model()`.
#[must_use]
pub fn get_prompt_context_for_model(model_id: Option<&str>) -> ImageModelPromptContext {
    let model = model_id.unwrap_or(DEFAULT_IMAGE_MODEL_ID);
    let model_lower = model.to_lowercase();

    // =========================================================================
    // T5-based models (natural language prompts)
    // =========================================================================

    // PixArt models
    if model_lower.contains("pixart") {
        let display_name = if model_lower.contains("sigma") {
            "PixArt-Sigma"
        } else {
            "PixArt-Alpha"
        };
        return ImageModelPromptContext {
            display_name: display_name.to_string(),
            family: "pixart".to_string(),
        };
    }

    // Hunyuan models (Tencent)
    if model_lower.contains("hunyuan") {
        let display_name = if model_lower.contains("3.0") || model_lower.contains("image") {
            "HunyuanImage 3.0"
        } else {
            "HunyuanDiT"
        };
        return ImageModelPromptContext {
            display_name: display_name.to_string(),
            family: "hunyuan".to_string(),
        };
    }

    // Kolors (Kwai)
    if model_lower.contains("kolors") {
        return ImageModelPromptContext {
            display_name: "Kolors".to_string(),
            family: "kolors".to_string(),
        };
    }

    // DeepFloyd IF
    if model_lower.contains("deepfloyd") || model_lower.contains("if-i-") {
        return ImageModelPromptContext {
            display_name: "DeepFloyd IF".to_string(),
            family: "deepfloyd".to_string(),
        };
    }

    // =========================================================================
    // CLIP-based models (tag-style prompts)
    // =========================================================================

    // SDXL
    if model_lower.contains("sdxl") || model_lower.contains("stable-diffusion-xl") {
        return ImageModelPromptContext {
            display_name: "Stable Diffusion XL".to_string(),
            family: "sdxl".to_string(),
        };
    }

    // Stable Cascade / Würstchen
    if model_lower.contains("cascade") || model_lower.contains("wuerstchen") {
        return ImageModelPromptContext {
            display_name: "Stable Cascade".to_string(),
            family: "cascade".to_string(),
        };
    }

    // SD 2.x models
    if model_lower.contains("stable-diffusion-2") {
        return ImageModelPromptContext {
            display_name: "Stable Diffusion 2.1".to_string(),
            family: "sd2".to_string(),
        };
    }

    // SD 1.5 and legacy models
    if model_lower.contains("stable-diffusion-v1-5")
        || model_lower.contains("sd-v1-5")
        || model_lower.contains("stable-diffusion-1")
        || model_lower.contains("compvis")
    {
        return ImageModelPromptContext {
            display_name: "Stable Diffusion 1.5".to_string(),
            family: "sd15".to_string(),
        };
    }

    // Kandinsky models
    if model_lower.contains("kandinsky") {
        let display_name = if model_lower.contains('3') {
            "Kandinsky 3.1"
        } else {
            "Kandinsky 2.2"
        };
        return ImageModelPromptContext {
            display_name: display_name.to_string(),
            family: "kandinsky".to_string(),
        };
    }

    // Default fallback (generic Stable Diffusion compatible)
    ImageModelPromptContext {
        display_name: "Stable Diffusion".to_string(),
        family: "stable-diffusion".to_string(),
    }
}
