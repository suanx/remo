//! Vision API provider implementations.
//!
//! Supports OpenAI GPT-4o, Anthropic Claude 3.5 Sonnet, and Ollama (e.g. llava)
//! as vision backends for describing images from URLs or base64-encoded data.

use serde::{Deserialize, Serialize};

use crate::config::MultimodalConfig;

// ---------------------------------------------------------------------------
// VisionProvider enum
// ---------------------------------------------------------------------------

/// Supported vision API providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VisionProvider {
    /// OpenAI GPT-4o Vision.
    OpenAI,
    /// Anthropic Claude 3.5 Sonnet.
    Anthropic,
    /// Ollama (local, e.g. llava).
    Ollama,
}

impl VisionProvider {
    /// Parse a provider name string (case-insensitive).
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "openai" => Some(Self::OpenAI),
            "anthropic" => Some(Self::Anthropic),
            "ollama" => Some(Self::Ollama),
            _ => None,
        }
    }

    /// Default model name for this provider.
    pub fn default_model(&self) -> &str {
        match self {
            Self::OpenAI => "gpt-4o",
            Self::Anthropic => "claude-3-5-sonnet-20241022",
            Self::Ollama => "llava",
        }
    }

    /// Default base URL for this provider.
    pub fn default_base_url(&self) -> &str {
        match self {
            Self::OpenAI => "https://api.openai.com/v1",
            Self::Anthropic => "https://api.anthropic.com",
            Self::Ollama => "http://localhost:11434",
        }
    }
}

// ---------------------------------------------------------------------------
// VisionProviderConfig
// ---------------------------------------------------------------------------

/// Configuration for a vision provider.
#[derive(Debug, Clone)]
pub struct VisionProviderConfig {
    /// The vision provider.
    pub provider: VisionProvider,
    /// API key (required for OpenAI and Anthropic, optional for Ollama).
    pub api_key: Option<String>,
    /// Model name (defaults to per-provider default).
    pub model: String,
    /// Base URL for API requests.
    pub base_url: String,
    /// Maximum tokens in the response.
    pub max_tokens: u32,
}

impl Default for VisionProviderConfig {
    fn default() -> Self {
        let provider = VisionProvider::OpenAI;
        Self {
            provider,
            api_key: None,
            model: provider.default_model().to_string(),
            base_url: provider.default_base_url().to_string(),
            max_tokens: 300,
        }
    }
}

// ---------------------------------------------------------------------------
// VisionClient
// ---------------------------------------------------------------------------

/// A client for interacting with vision API providers.
///
/// Supports OpenAI, Anthropic, and Ollama backends for describing images
/// from URLs or base64-encoded data.
///
/// # Example
///
/// ```ignore
/// let client = VisionClient::from_config(&config).unwrap();
/// let description = client.describe_image("https://example.com/photo.jpg", Some("what color is the car?")).await?;
/// ```
pub struct VisionClient {
    config: VisionProviderConfig,
    http_client: reqwest::Client,
}

impl VisionClient {
    /// Create a new vision client with the given configuration.
    pub fn new(config: VisionProviderConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }

    /// Attempt to create a vision client from the multimodal extension config.
    ///
    /// Returns `None` if no `vision_provider` is configured or if the provider
    /// name is unrecognised.
    pub fn from_config(config: &MultimodalConfig) -> Option<Self> {
        let provider_str = config.vision_provider.as_deref()?;
        let provider = VisionProvider::from_str(provider_str)?;

        let model = config
            .vision_model
            .clone()
            .unwrap_or_else(|| provider.default_model().to_string());

        let base_url = config
            .vision_base_url
            .clone()
            .unwrap_or_else(|| provider.default_base_url().to_string());

        let max_tokens = config.vision_max_tokens.unwrap_or(300);

        Some(Self::new(VisionProviderConfig {
            provider,
            api_key: config.vision_api_key.clone(),
            model,
            base_url,
            max_tokens,
        }))
    }

    /// Describe an image from a URL or base64-encoded source.
    ///
    /// # Arguments
    ///
    /// * `source` – Image URL (`http://` / `https://`) or base64-encoded data
    ///   (raw base64 string or a `data:` URI).
    /// * `hint` – Optional hint about what to look for in the image.
    ///
    /// # Returns
    ///
    /// A text description of the image returned by the configured provider.
    pub async fn describe_image(
        &self,
        source: &str,
        hint: Option<&str>,
    ) -> Result<String, String> {
        let prompt = match hint {
            Some(h) => format!("Please describe this image in detail. Focus on: {h}"),
            None => "Please describe this image in detail.".to_string(),
        };

        match self.config.provider {
            VisionProvider::OpenAI => self.describe_openai(source, &prompt).await,
            VisionProvider::Anthropic => self.describe_anthropic(source, &prompt).await,
            VisionProvider::Ollama => self.describe_ollama(source, &prompt).await,
        }
    }

    // ── Source classification ───────────────────────────────────────────

    /// Classify an image source string.
    ///
    /// Returns a tuple `(kind, payload)` where `kind` is one of `"url"`,
    /// `"data_uri"` or `"base64"`, and `payload` is the relevant data portion.
    fn classify_source(source: &str) -> (&str, &str) {
        let trimmed = source.trim();
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            ("url", trimmed)
        } else if let Some(rest) = trimmed.strip_prefix("data:") {
            if let Some((_mime, b64_data)) = rest.split_once(";base64,") {
                ("data_uri", b64_data)
            } else {
                ("base64", trimmed)
            }
        } else {
            ("base64", trimmed)
        }
    }

    /// Extract the MIME type from a `data:` URI prefix.
    ///
    /// Returns e.g. `"image/png"` from `"data:image/png;base64,iVBOR…"`.
    fn extract_mime_from_data_uri(data_uri: &str) -> Option<&str> {
        let after_data = data_uri.strip_prefix("data:")?;
        after_data.split(';').next()
    }

    // ── OpenAI provider ──────────────────────────────────────────────────

    async fn describe_openai(&self, source: &str, prompt: &str) -> Result<String, String> {
        let url = format!(
            "{}/chat/completions",
            self.config.base_url.trim_end_matches('/')
        );

        let api_key = self
            .config
            .api_key
            .as_deref()
            .ok_or_else(|| "OpenAI API key is not configured".to_string())?;

        let image_content = self.build_openai_image_content(source);

        let body = serde_json::json!({
            "model": self.config.model,
            "messages": [{
                "role": "user",
                "content": [
                    { "type": "text", "text": prompt },
                    image_content
                ]
            }],
            "max_tokens": self.config.max_tokens
        });

        let resp = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("OpenAI request failed: {e}"))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read OpenAI response body: {e}"))?;

        if !status.is_success() {
            return Err(format!("OpenAI API error (HTTP {status}): {text}"));
        }

        let json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("Invalid JSON from OpenAI: {e}"))?;

        json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.trim().to_string())
            .ok_or_else(|| {
                let snippet = if text.len() > 200 {
                    format!("{}...", &text[..200])
                } else {
                    text.clone()
                };
                format!("Unexpected OpenAI response format: {snippet}")
            })
    }

    /// Build the image content part for an OpenAI request.
    fn build_openai_image_content(&self, source: &str) -> serde_json::Value {
        let (kind, _data) = Self::classify_source(source);
        match kind {
            "url" => serde_json::json!({
                "type": "image_url",
                "image_url": { "url": source }
            }),
            "data_uri" => serde_json::json!({
                "type": "image_url",
                "image_url": { "url": source }
            }),
            _ => {
                // Raw base64 — wrap as data URI with JPEG default
                serde_json::json!({
                    "type": "image_url",
                    "image_url": { "url": format!("data:image/jpeg;base64,{}", source) }
                })
            }
        }
    }

    // ── Anthropic provider ───────────────────────────────────────────────

    async fn describe_anthropic(&self, source: &str, prompt: &str) -> Result<String, String> {
        let url = format!(
            "{}/v1/messages",
            self.config.base_url.trim_end_matches('/')
        );

        let api_key = self
            .config
            .api_key
            .as_deref()
            .ok_or_else(|| "Anthropic API key is not configured".to_string())?;

        let image_content = self.build_anthropic_image_content(source);

        let body = serde_json::json!({
            "model": self.config.model,
            "max_tokens": self.config.max_tokens,
            "messages": [{
                "role": "user",
                "content": [
                    { "type": "text", "text": prompt },
                    image_content
                ]
            }]
        });

        let resp = self
            .http_client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Anthropic request failed: {e}"))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read Anthropic response body: {e}"))?;

        if !status.is_success() {
            return Err(format!("Anthropic API error (HTTP {status}): {text}"));
        }

        let json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("Invalid JSON from Anthropic: {e}"))?;

        // Anthropic returns content as an array of content blocks
        let content = json["content"].as_array().ok_or_else(|| {
            format!("Unexpected Anthropic response (missing content array): {text}")
        })?;

        for block in content {
            if block["type"] == "text" {
                if let Some(text) = block["text"].as_str() {
                    return Ok(text.trim().to_string());
                }
            }
        }

        Err(format!("No text content in Anthropic response: {text}"))
    }

    /// Build the image content part for an Anthropic request.
    fn build_anthropic_image_content(&self, source: &str) -> serde_json::Value {
        let (kind, payload) = Self::classify_source(source);
        match kind {
            "url" => serde_json::json!({
                "type": "image",
                "source": { "type": "url", "url": source }
            }),
            "data_uri" => {
                let media_type =
                    Self::extract_mime_from_data_uri(source).unwrap_or("image/jpeg");
                // payload is the base64 data portion from classify_source
                serde_json::json!({
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": media_type,
                        "data": payload
                    }
                })
            }
            _ => {
                // Raw base64 — assume JPEG
                serde_json::json!({
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": "image/jpeg",
                        "data": payload
                    }
                })
            }
        }
    }

    // ── Ollama provider ──────────────────────────────────────────────────

    async fn describe_ollama(&self, source: &str, prompt: &str) -> Result<String, String> {
        let url = format!(
            "{}/api/chat",
            self.config.base_url.trim_end_matches('/')
        );

        // Ollama requires images as base64-encoded strings. If the source is
        // a URL, download the image and encode it first.
        let base64_data = if source.starts_with("http://") || source.starts_with("https://") {
            self.fetch_image_as_base64(source).await?
        } else {
            let (_kind, data) = Self::classify_source(source);
            data.to_string()
        };

        let body = serde_json::json!({
            "model": self.config.model,
            "messages": [{
                "role": "user",
                "content": prompt,
                "images": [base64_data]
            }],
            "stream": false
        });

        let resp = self
            .http_client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Ollama request failed: {e}"))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read Ollama response body: {e}"))?;

        if !status.is_success() {
            return Err(format!("Ollama API error (HTTP {status}): {text}"));
        }

        let json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("Invalid JSON from Ollama: {e}"))?;

        json["message"]["content"]
            .as_str()
            .map(|s| s.trim().to_string())
            .ok_or_else(|| {
                let snippet = if text.len() > 200 {
                    format!("{}...", &text[..200])
                } else {
                    text.clone()
                };
                format!("Unexpected Ollama response format: {snippet}")
            })
    }

    /// Fetch an image from a URL and encode it as a base64 string.
    async fn fetch_image_as_base64(&self, url: &str) -> Result<String, String> {
        let resp = self
            .http_client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch image from {url}: {e}"))?;

        let status = resp.status();
        if !status.is_success() {
            return Err(format!(
                "Failed to fetch image (HTTP {status}): {url}"
            ));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| format!("Failed to read image bytes: {e}"))?;

        use base64::Engine;
        Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vision_provider_from_str() {
        assert_eq!(
            VisionProvider::from_str("openai"),
            Some(VisionProvider::OpenAI)
        );
        assert_eq!(
            VisionProvider::from_str("OpenAI"),
            Some(VisionProvider::OpenAI)
        );
        assert_eq!(
            VisionProvider::from_str("anthropic"),
            Some(VisionProvider::Anthropic)
        );
        assert_eq!(
            VisionProvider::from_str("OLLAMA"),
            Some(VisionProvider::Ollama)
        );
        assert_eq!(VisionProvider::from_str("unknown"), None);
        assert_eq!(VisionProvider::from_str(""), None);
    }

    #[test]
    fn vision_provider_default_model() {
        assert_eq!(VisionProvider::OpenAI.default_model(), "gpt-4o");
        assert_eq!(
            VisionProvider::Anthropic.default_model(),
            "claude-3-5-sonnet-20241022"
        );
        assert_eq!(VisionProvider::Ollama.default_model(), "llava");
    }

    #[test]
    fn vision_provider_default_base_url() {
        assert_eq!(
            VisionProvider::OpenAI.default_base_url(),
            "https://api.openai.com/v1"
        );
        assert_eq!(
            VisionProvider::Anthropic.default_base_url(),
            "https://api.anthropic.com"
        );
        assert_eq!(
            VisionProvider::Ollama.default_base_url(),
            "http://localhost:11434"
        );
    }

    #[test]
    fn classify_source_url() {
        assert_eq!(
            VisionClient::classify_source("https://example.com/img.png"),
            ("url", "https://example.com/img.png")
        );
        assert_eq!(
            VisionClient::classify_source("http://x.co/a.jpg"),
            ("url", "http://x.co/a.jpg")
        );
    }

    #[test]
    fn classify_source_data_uri() {
        let (kind, data) =
            VisionClient::classify_source("data:image/png;base64,iVBORw0KGgo");
        assert_eq!(kind, "data_uri");
        assert_eq!(data, "iVBORw0KGgo");
    }

    #[test]
    fn classify_source_raw_base64() {
        let (kind, data) =
            VisionClient::classify_source("iVBORw0KGgoAAAANSUhEUg");
        assert_eq!(kind, "base64");
        assert_eq!(data, "iVBORw0KGgoAAAANSUhEUg");
    }

    #[test]
    fn classify_source_empty_string() {
        let (kind, data) = VisionClient::classify_source("");
        assert_eq!(kind, "base64");
        assert_eq!(data, "");
    }

    #[test]
    fn extract_mime_from_data_uri() {
        assert_eq!(
            VisionClient::extract_mime_from_data_uri("data:image/png;base64,iVBOR"),
            Some("image/png")
        );
        assert_eq!(
            VisionClient::extract_mime_from_data_uri("data:image/jpeg;base64,/9j/4"),
            Some("image/jpeg")
        );
        assert_eq!(
            VisionClient::extract_mime_from_data_uri("data:image/webp;base64,UklG"),
            Some("image/webp")
        );
        assert_eq!(
            VisionClient::extract_mime_from_data_uri("not-a-data-uri"),
            None
        );
    }

    #[test]
    fn config_default_values() {
        let cfg = VisionProviderConfig::default();
        assert_eq!(cfg.provider, VisionProvider::OpenAI);
        assert_eq!(cfg.model, "gpt-4o");
        assert_eq!(cfg.base_url, "https://api.openai.com/v1");
        assert_eq!(cfg.max_tokens, 300);
        assert!(cfg.api_key.is_none());
    }

    #[test]
    fn from_config_returns_none_when_no_provider() {
        let config = MultimodalConfig::default(); // vision_provider = None
        assert!(VisionClient::from_config(&config).is_none());
    }

    #[test]
    fn from_config_returns_none_for_unknown_provider() {
        let mut config = MultimodalConfig::default();
        config.vision_provider = Some("invalid".to_string());
        assert!(VisionClient::from_config(&config).is_none());
    }

    #[test]
    fn from_config_uses_custom_values() {
        let mut config = MultimodalConfig::default();
        config.vision_provider = Some("anthropic".to_string());
        config.vision_model = Some("claude-3-opus".to_string());
        config.vision_api_key = Some("sk-ant-xxx".to_string());
        config.vision_base_url = Some("https://custom.anthropic.com".to_string());
        config.vision_max_tokens = Some(500);

        let client = VisionClient::from_config(&config).unwrap();
        assert_eq!(client.config.provider, VisionProvider::Anthropic);
        assert_eq!(client.config.model, "claude-3-opus");
        assert_eq!(client.config.api_key.as_deref(), Some("sk-ant-xxx"));
        assert_eq!(client.config.base_url, "https://custom.anthropic.com");
        assert_eq!(client.config.max_tokens, 500);
    }
}
