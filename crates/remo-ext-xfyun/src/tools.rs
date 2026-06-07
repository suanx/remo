//! Typed tools for 讯飞星辰 MaaS Embedding & Rerank APIs.
//!
//! Provides two [`TypedTool`] implementations:
//! - [`GetEmbeddingTool`] — 将文本转换为向量表示
//! - [`RerankDocumentsTool`] — 对文档列表根据查询进行相关性重排序

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing;

use remo_runtime_contract::contract::tool::{
    ToolCallContext, ToolError, ToolOutput, ToolResult, TypedTool,
};

use crate::config::XfyunConfigKey;
use serde_json::json;
use crate::config::XfyunConfig;


// ---------------------------------------------------------------------------
// GetEmbeddingTool
// ---------------------------------------------------------------------------

/// Arguments for [`GetEmbeddingTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetEmbeddingArgs {
    /// 要向量化的文本（或 JSON array 字符串表示多段文本）
    pub input: String,
    /// 模型 ID，默认用配置中的 embedding_model
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// 返回格式，默认 float
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,
}

/// Tool that converts text into vector embeddings using the Xfyun MaaS Embedding API.
pub struct GetEmbeddingTool;

impl GetEmbeddingTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GetEmbeddingTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TypedTool for GetEmbeddingTool {
    type Args = GetEmbeddingArgs;

    fn tool_id(&self) -> &str {
        "xfyun:get_embedding"
    }

    fn name(&self) -> &str {
        "Xfyun Get Embedding"
    }

    fn description(&self) -> &str {
        "将文本转换为向量表示，使用讯飞星辰 MaaS Embedding API"
    }

    fn category(&self) -> Option<&str> {
        Some("xfyun")
    }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        // 1. Read config
        let config = ctx
            .agent_spec
            .config::<XfyunConfigKey>()
            .map_err(|e| {
                ToolError::Internal(format!("读取 Xfyun 配置失败: {e}"))
            })?;

        let api_key = config.api_key.clone().ok_or_else(|| {
            ToolError::InvalidArguments("Xfyun API Key 未配置，请在 agent 配置中设置 api_key".into())
        })?;

        let base_url = config.effective_base_url();
        let model = args.model.unwrap_or_else(|| config.embedding_model.clone());
        let encoding_format = args.encoding_format.unwrap_or_else(|| "float".to_string());

        // 2. Parse input — try as JSON array first, fall back to plain string
        let input_value: serde_json::Value = serde_json::from_str(&args.input)
            .unwrap_or_else(|_| serde_json::Value::String(args.input.clone()));

        // 3. Build request body
        let body = serde_json::json!({
            "model": model,
            "input": input_value,
            "encoding_format": encoding_format,
        });

        let url = format!("{}/embeddings", base_url);

        tracing::info!(
            target: "remo_ext_xfyun::tools",
            tool = "get_embedding",
            model = %model,
            base_url = %base_url,
            input_len = args.input.len(),
            "请求 Embedding API"
        );

        // 4. Send POST request
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                ToolError::ExecutionFailed(format!("HTTP 请求失败: {e}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(ToolError::ExecutionFailed(format!(
                "Embedding API 返回错误 (HTTP {status}): {error_text}"
            )));
        }

        let result: serde_json::Value = response.json().await.map_err(|e| {
            ToolError::ExecutionFailed(format!("JSON 解析失败: {e}"))
        })?;

        // 5. Extract and format embedding data
        let mut output_data = serde_json::json!({
            "model": model,
            "object": result.get("object"),
        });

        // Extract usage info
        if let Some(usage) = result.get("usage") {
            output_data["usage"] = usage.clone();
        }

        // Extract embedding vectors (show first 5 dimensions + total dims)
        if let Some(data) = result.get("data").and_then(|d| d.as_array()) {
            let mut embeddings_info = Vec::new();
            for entry in data {
                if let Some(embedding) = entry.get("embedding") {
                    let dims = match embedding {
                        serde_json::Value::Array(arr) => arr.len(),
                        _ => 0,
                    };
                    let preview: Vec<serde_json::Value> = embedding
                        .as_array()
                        .map(|arr| arr.iter().take(5).cloned().collect())
                        .unwrap_or_default();

                    let index = entry.get("index").cloned().unwrap_or(serde_json::Value::Null);
                    embeddings_info.push(serde_json::json!({
                        "index": index,
                        "embedding_preview": preview,
                        "total_dimensions": dims,
                        "preview_dimensions": preview.len(),
                    }));
                }
            }
            output_data["embeddings"] = serde_json::Value::Array(embeddings_info);
            output_data["count"] = serde_json::json!(data.len());
        }

        tracing::info!(
            target: "remo_ext_xfyun::tools",
            tool = "get_embedding",
            model = %model,
            status = "success",
            "Embedding API 调用成功"
        );

        Ok(ToolResult::success("xfyun:get_embedding", output_data).into())
    }
}

// ---------------------------------------------------------------------------
// RerankDocumentsTool
// ---------------------------------------------------------------------------

/// Arguments for [`RerankDocumentsTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RerankDocumentsArgs {
    /// 查询语句
    pub query: String,
    /// 需要重排序的文档列表
    pub documents: Vec<String>,
    /// 模型 ID，默认用配置中的 rerank_model
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Tool that reranks a list of documents by relevance to a query using Xfyun MaaS Rerank API.
pub struct RerankDocumentsTool;

impl RerankDocumentsTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RerankDocumentsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TypedTool for RerankDocumentsTool {
    type Args = RerankDocumentsArgs;

    fn tool_id(&self) -> &str {
        "xfyun:rerank_documents"
    }

    fn name(&self) -> &str {
        "Xfyun Rerank Documents"
    }

    fn description(&self) -> &str {
        "对文档列表根据查询进行相关性重排序，使用讯飞星辰 MaaS Rerank API"
    }

    fn category(&self) -> Option<&str> {
        Some("xfyun")
    }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        // 1. Read config
        let config = ctx
            .agent_spec
            .config::<XfyunConfigKey>()
            .map_err(|e| {
                ToolError::Internal(format!("读取 Xfyun 配置失败: {e}"))
            })?;

        let api_key = config.api_key.clone().ok_or_else(|| {
            ToolError::InvalidArguments("Xfyun API Key 未配置，请在 agent 配置中设置 api_key".into())
        })?;

        let base_url = config.effective_base_url();
        let model = args.model.unwrap_or_else(|| config.rerank_model.clone());

        if args.documents.is_empty() {
            return Err(ToolError::InvalidArguments(
                "文档列表不能为空，请提供至少一个文档".into(),
            ));
        }

        // 2. Build request body
        let body = serde_json::json!({
            "model": model,
            "query": args.query,
            "documents": args.documents,
        });

        let url = format!("{}/rerank", base_url);

        tracing::info!(
            target: "remo_ext_xfyun::tools",
            tool = "rerank_documents",
            model = %model,
            base_url = %base_url,
            query_len = args.query.len(),
            doc_count = args.documents.len(),
            "请求 Rerank API"
        );

        // 3. Send POST request
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                ToolError::ExecutionFailed(format!("HTTP 请求失败: {e}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(ToolError::ExecutionFailed(format!(
                "Rerank API 返回错误 (HTTP {status}): {error_text}"
            )));
        }

        let result: serde_json::Value = response.json().await.map_err(|e| {
            ToolError::ExecutionFailed(format!("JSON 解析失败: {e}"))
        })?;

        // 4. Extract and sort results by relevance_score descending
        let mut output_data = serde_json::json!({
            "model": model,
            "query": args.query,
        });

        if let Some(usage) = result.get("usage") {
            output_data["usage"] = usage.clone();
        }

        if let Some(results) = result.get("results").and_then(|r| r.as_array()) {
            let mut sorted: Vec<serde_json::Value> = results.clone();
            sorted.sort_by(|a, b| {
                let score_a = a.get("relevance_score").and_then(|s| s.as_f64()).unwrap_or(0.0);
                let score_b = b.get("relevance_score").and_then(|s| s.as_f64()).unwrap_or(0.0);
                score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
            });

            // Attach original document content to each result
            let enriched: Vec<serde_json::Value> = sorted
                .iter()
                .map(|item| {
                    let idx = item.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;
                    let original_doc = args.documents.get(idx).cloned().unwrap_or_default();
                    let mut enriched = item.clone();
                    enriched["document"] = serde_json::json!(original_doc);
                    enriched
                })
                .collect();

            output_data["results"] = serde_json::Value::Array(enriched);
            output_data["count"] = serde_json::json!(enriched.len());
        }

        tracing::info!(
            target: "remo_ext_xfyun::tools",
            tool = "rerank_documents",
            model = %model,
            doc_count = args.documents.len(),
            status = "success",
            "Rerank API 调用成功"
        );

        Ok(ToolResult::success("xfyun:rerank_documents", output_data).into())
    }
}

// ---------------------------------------------------------------------------
// XfyunImageGenTool
// ---------------------------------------------------------------------------

/// Arguments for [`XfyunImageGenTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct XfyunImageGenArgs {
    /// 文本提示词，描述要生成的图片
    pub prompt: String,
    /// 负面提示词（可选）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    /// 模型 domain ID（从星辰网页获取，可选，默认从配置读取）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// 图片宽度（可选，默认 768）
    #[serde(default)]
    pub width: Option<u32>,
    /// 图片高度（可选，默认 768）
    #[serde(default)]
    pub height: Option<u32>,
    /// 生成步数 1-50（可选，默认 20）
    #[serde(default)]
    pub steps: Option<u32>,
    /// 提示词相关度 0-20（可选，默认 5.0）
    #[serde(default)]
    pub guidance_scale: Option<f64>,
    /// 随机种子（可选）
    #[serde(default)]
    pub seed: Option<u32>,
    /// 调度器（可选，DPM++ 2M Karras / DPM++ SDE Karras / DDIM / Euler a / Euler）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheduler: Option<String>,
    /// 自定义 TTI 端点（可选，覆盖配置）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    /// 用户 ID（可选）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>,
    /// patch_id（可选，非全量训练模型需要）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patch_id: Option<String>,
}

/// Tool that generates images using 讯飞星辰 MaaS TTI (Text-To-Image) API.
///
/// Supports 星火大模型图片生成 and Kolors 模型。
/// 请求地址: https://maas-api.cn-huabei-1.xf-yun.com/v2.1/tti
pub struct XfyunImageGenTool;

#[async_trait]
impl TypedTool for XfyunImageGenTool {
    type Args = XfyunImageGenArgs;

    fn tool_id(&self) -> &str { "xfyun:generate_image" }
    fn name(&self) -> &str { "Xfyun Generate Image" }
    fn description(&self) -> &str {
        "Generate an image using 讯飞星辰 MaaS TTI API. Supports 星火大模型 and Kolors models. \
         Returns a base64-encoded image. Requires app_id and api_key configured."
    }
    fn category(&self) -> Option<&str> { Some("xfyun") }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let config: XfyunConfig = ctx
            .agent_spec
            .config::<XfyunConfigKey>()
            .map_err(|e| ToolError::Internal(format!("Failed to read config: {e}")))?;

        let app_id = config.app_id.as_ref()
            .ok_or_else(|| ToolError::ExecutionFailed(
                "app_id not configured. Set it in the xfyun config section (required for TTI API).".into()
            ))?;

        let api_key = config.api_key.as_ref()
            .ok_or_else(|| ToolError::ExecutionFailed(
                "api_key not configured. Set it in the xfyun config section.".into()
            ))?;

        let endpoint = args.endpoint.clone()
            .unwrap_or(config.tti_endpoint.clone());

        let model = args.model.clone()
            .unwrap_or_else(|| {
                if config.tti_model.is_empty() {
                    "".to_string()
                } else {
                    config.tti_model.clone()
                }
            });

        let width = args.width.unwrap_or(config.tti_width);
        let height = args.height.unwrap_or(config.tti_height);
        let steps = args.steps.unwrap_or(config.tti_steps);
        let guidance = args.guidance_scale.unwrap_or(config.tti_guidance_scale);
        let seed = args.seed.unwrap_or(config.tti_seed);
        let scheduler = args.scheduler.clone().unwrap_or(config.tti_scheduler.clone());

        // Build the xfyun TTI request body
        let mut header = json!({
            "app_id": app_id,
        });
        if let Some(ref uid) = args.uid {
            header["uid"] = json!(uid);
        }
        if let Some(ref patch_id) = args.patch_id {
            header["patch_id"] = json!(patch_id);
        }

        let mut body = json!({
            "header": header,
            "parameter": {
                "chat": {
                    "domain": model,
                    "width": width,
                    "height": height,
                    "seed": seed,
                    "num_inference_steps": steps,
                    "guidance_scale": guidance,
                    "scheduler": scheduler,
                }
            },
            "payload": {
                "message": {
                    "text": [{
                        "role": "user",
                        "content": &args.prompt,
                    }]
                }
            }
        });

        // Only add negative_prompts if present (avoid sending null)
        if let Some(ref neg) = args.negative_prompt {
            body["payload"]["negative_prompts"] = json!({
                "text": neg,
            });
        }

        let client = reqwest::Client::new();
        let resp = client
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("HTTP request failed: {e}")))?;

        let status = resp.status();
        let response_body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse response: {e}")))?;

        // Check for API-level error
        if !status.is_success() {
            let code = response_body["header"]["code"].as_i64().unwrap_or(-1);
            let message = response_body["header"]["message"].as_str().unwrap_or("unknown error");
            return Err(ToolError::ExecutionFailed(format!(
                "Xfyun TTI API returned HTTP {status} (code {code}): {message}"
            )));
        }

        // Check header code
        let header_code = response_body["header"]["code"].as_i64().unwrap_or(-1);
        if header_code != 0 {
            let msg = response_body["header"]["message"].as_str().unwrap_or("unknown error");
            return Err(ToolError::ExecutionFailed(format!(
                "Xfyun TTI API error (code {header_code}): {msg}"
            )));
        }

        // Extract base64 image
        let image_base64 = response_body["payload"]["choices"]["text"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|item| item["content"].as_str())
            .map(|s| s.to_string());

        let sid = response_body["header"]["sid"].as_str().unwrap_or("");

        tracing::info!(
            target: "remo::xfyun::tti",
            sid = %sid,
            width = width,
            height = height,
            has_image = image_base64.is_some(),
            "Image generated via Xfyun TTI"
        );

        Ok(ToolResult::success("xfyun:generate_image", json!({
            "status": if image_base64.is_some() { "success" } else { "no_image" },
            "sid": sid,
            "image_base64": image_base64,
            "image_prefix": "data:image/png;base64,",
            "width": width,
            "height": height,
            "model": model,
            "prompt": args.prompt,
        })).into())
    }
}


// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_embedding_tool_descriptor() {
        let tool = GetEmbeddingTool::new();
        assert_eq!(tool.tool_id(), "xfyun:get_embedding");
        assert_eq!(tool.name(), "Xfyun Get Embedding");
        assert!(tool.description().contains("向量"));
        assert_eq!(tool.category(), Some("xfyun"));
    }

    #[test]
    fn rerank_documents_tool_descriptor() {
        let tool = RerankDocumentsTool::new();
        assert_eq!(tool.tool_id(), "xfyun:rerank_documents");
        assert_eq!(tool.name(), "Xfyun Rerank Documents");
        assert!(tool.description().contains("重排序"));
        assert_eq!(tool.category(), Some("xfyun"));
    }

    #[test]
    fn get_embedding_args_defaults() {
        let args = GetEmbeddingArgs {
            input: "hello world".to_string(),
            model: None,
            encoding_format: None,
        };
        assert_eq!(args.input, "hello world");
        assert!(args.model.is_none());
        assert!(args.encoding_format.is_none());
    }

    #[test]
    fn rerank_documents_args_defaults() {
        let args = RerankDocumentsArgs {
            query: "test query".to_string(),
            documents: vec!["doc1".to_string(), "doc2".to_string()],
            model: None,
        };
        assert_eq!(args.query, "test query");
        assert_eq!(args.documents.len(), 2);
        assert!(args.model.is_none());
    }

    #[test]
    fn xfyun_image_gen_tool_descriptor() {
        let tool = XfyunImageGenTool;
        assert_eq!(tool.tool_id(), "xfyun:generate_image");
        assert_eq!(tool.category(), Some("xfyun"));
    }

    #[test]
    fn xfyun_image_gen_args_defaults() {
        let args = XfyunImageGenArgs {
            prompt: "a cat".to_string(),
            negative_prompt: None,
            model: None,
            width: None,
            height: None,
            steps: None,
            guidance_scale: None,
            seed: None,
            scheduler: None,
            endpoint: None,
            uid: None,
            patch_id: None,
        };
        assert_eq!(args.prompt, "a cat");
    }

}
