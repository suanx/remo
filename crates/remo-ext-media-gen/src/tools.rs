//! Tools for image and video generation.

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use remo_runtime_contract::contract::tool::{
    ToolCallContext, ToolError, ToolOutput, ToolResult, TypedTool,
};
use remo_runtime_contract::PluginConfigKey;

use crate::config::{MediaGenConfig, MediaGenConfigKey};

// ---------------------------------------------------------------------------
// GenerateImageTool
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GenerateImageArgs {
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default)]
    pub n: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

pub struct GenerateImageTool;

#[async_trait]
impl TypedTool for GenerateImageTool {
    type Args = GenerateImageArgs;

    fn tool_id(&self) -> &str { "media:generate_image" }
    fn name(&self) -> &str { "Generate Image" }
    fn description(&self) -> &str {
        "Generate an image from a text prompt. Supports OpenAI DALL-E 3, Agnes AI Image, and OpenAI-compatible APIs."
    }
    fn category(&self) -> Option<&str> { Some("media") }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let config: MediaGenConfig = ctx
            .agent_spec
            .config::<MediaGenConfigKey>()
            .map_err(|e| ToolError::Internal(format!("Failed to read config: {e}")))?;

        let provider = args.provider.as_deref().unwrap_or(&config.default_image_provider);
        let model = args.model.as_deref().unwrap_or(&config.default_image_model);
        let size = args.size.as_deref().unwrap_or(&config.default_image_size);
        let n = args.n.unwrap_or(config.n).min(4).max(1);
        let quality = args.quality.as_deref().unwrap_or(&config.image_quality);

        // Resolve base_url and api_key
        let (base_url, api_key) = match provider {
            "openai" => {
                let key = config.openai_api_key.clone().or_else(|| args.api_key.clone())
                    .ok_or_else(|| ToolError::ExecutionFailed("OpenAI API key not configured.".into()))?;
                (args.base_url.clone().unwrap_or_else(|| "https://api.openai.com/v1".into()), key)
            }
            "agnes" => {
                let key = config.agnes_api_key.clone().or_else(|| args.api_key.clone())
                    .ok_or_else(|| ToolError::ExecutionFailed("Agnes AI API key not configured.".into()))?;
                (args.base_url.clone().unwrap_or(config.agnes_base_url.clone()), key)
            }
            _ => {
                let key = args.api_key.clone().or_else(|| config.openai_api_key.clone()).or_else(|| config.agnes_api_key.clone())
                    .ok_or_else(|| ToolError::ExecutionFailed("API key required.".into()))?;
                let url = args.base_url.clone().or(config.custom_base_url.clone()).unwrap_or_else(|| "https://api.openai.com/v1".into());
                (url, key)
            }
        };

        let endpoint = format!("{}/images/generations", base_url.trim_end_matches('/'));
        let client = reqwest::Client::new();
        let mut payload = json!({ "model": model, "prompt": args.prompt, "n": n, "size": size });
        if provider == "openai" && model.contains("dall-e-3") {
            payload["quality"] = json!(quality);
            payload["response_format"] = json!("b64_json");
        }

        let resp = client.post(&endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send().await
            .map_err(|e| ToolError::ExecutionFailed(format!("HTTP request failed: {e}")))?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse response: {e}")))?;

        if !status.is_success() {
            let err_msg = body.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()).unwrap_or("unknown");
            return Err(ToolError::ExecutionFailed(format!("Image API returned HTTP {status}: {err_msg}")));
        }

        let images = body.get("data").and_then(|d| d.as_array())
            .map(|arr| arr.iter().map(|item| json!({
                "b64_json": item.get("b64_json").and_then(|b| b.as_str()),
                "url": item.get("url").and_then(|u| u.as_str()),
                "revised_prompt": item.get("revised_prompt").and_then(|r| r.as_str()),
            })).collect::<Vec<_>>())
            .unwrap_or_default();

        tracing::info!(target: "remo::media::image", provider = %provider, model = %model, count = images.len(), "Image generated");

        Ok(ToolResult::success("media:generate_image", json!({
            "status": "success", "provider": provider, "model": model,
            "images": images, "image_count": images.len(),
        })).into())
    }
}

// ---------------------------------------------------------------------------
// GenerateVideoTool
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GenerateVideoArgs {
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default)]
    pub wait: bool,
}

pub struct GenerateVideoTool;

#[async_trait]
impl TypedTool for GenerateVideoTool {
    type Args = GenerateVideoArgs;

    fn tool_id(&self) -> &str { "media:generate_video" }
    fn name(&self) -> &str { "Generate Video" }
    fn description(&self) -> &str {
        "Generate a video from a text prompt. Supports Agnes AI Video and OpenAI-compatible video APIs."
    }
    fn category(&self) -> Option<&str> { Some("media") }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let config: MediaGenConfig = ctx
            .agent_spec
            .config::<MediaGenConfigKey>()
            .map_err(|e| ToolError::Internal(format!("Failed to read config: {e}")))?;

        let provider = args.provider.as_deref().unwrap_or(&config.default_video_provider);
        let model = args.model.as_deref().unwrap_or(&config.default_video_model);

        let (base_url, api_key) = match provider {
            "agnes" => {
                let key = config.agnes_api_key.clone().or_else(|| args.api_key.clone())
                    .ok_or_else(|| ToolError::ExecutionFailed("Agnes AI API key not configured.".into()))?;
                (args.base_url.clone().unwrap_or(config.agnes_base_url.clone()), key)
            }
            _ => {
                let url = args.base_url.clone().or(config.custom_base_url.clone())
                    .ok_or_else(|| ToolError::ExecutionFailed("Base URL required for video API.".into()))?;
                let key = args.api_key.clone().or_else(|| config.openai_api_key.clone())
                    .ok_or_else(|| ToolError::ExecutionFailed("API key required.".into()))?;
                (url, key)
            }
        };

        let endpoint = format!("{}/video/generations", base_url.trim_end_matches('/'));
        let client = reqwest::Client::new();
        let mut payload = json!({ "model": model, "prompt": args.prompt });
        if let Some(dur) = args.duration { payload["duration"] = json!(dur); }

        let resp = client.post(&endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send().await
            .map_err(|e| ToolError::ExecutionFailed(format!("HTTP request failed: {e}")))?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse response: {e}")))?;

        if !status.is_success() {
            let err_msg = body.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()).unwrap_or("unknown");
            return Err(ToolError::ExecutionFailed(format!("Video API returned HTTP {status}: {err_msg}")));
        }

        let task_id = body.get("id").or_else(|| body.get("task_id")).and_then(|v| v.as_str());
        let video_data = body.get("data").and_then(|d| d.as_array());

        tracing::info!(target: "remo::media::video", provider = %provider, model = %model, "Video generation submitted");

        Ok(ToolResult::success("media:generate_video", json!({
            "status": if task_id.is_some() && video_data.is_none() { "processing" } else { "completed" },
            "provider": provider, "model": model,
            "task_id": task_id, "data": video_data,
        })).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generate_image_tool_descriptor() {
        let tool = GenerateImageTool;
        assert_eq!(tool.tool_id(), "media:generate_image");
        assert_eq!(tool.category(), Some("media"));
    }
    #[test]
    fn generate_video_tool_descriptor() {
        let tool = GenerateVideoTool;
        assert_eq!(tool.tool_id(), "media:generate_video");
        assert_eq!(tool.category(), Some("media"));
    }
}
