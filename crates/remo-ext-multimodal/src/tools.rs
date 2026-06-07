//! Tool implementations for multimodal file processing and image description.
//!
//! Provides three [`TypedTool`] implementations:
//! - [`ParseFileTool`] for extracting text content from files of various formats.
//! - [`DescribeImageTool`] for describing a single image via a vision API.
//! - [`AnalyzeImageTool`] for batch-analyzing multiple images.

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use remo_runtime_contract::contract::tool::{
    ToolCallContext, ToolError, ToolOutput, ToolResult, TypedTool,
};

use crate::config::MultimodalConfigKey;
use crate::file_parser::FileParser;
use crate::vision::VisionClient;

// ---------------------------------------------------------------------------
// ParseFileTool
// ---------------------------------------------------------------------------

/// Arguments for [`ParseFileTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParseFileArgs {
    /// Absolute or relative path to the file to parse.
    pub file_path: String,
    /// Optional format override (e.g. `"csv"`, `"json"`, `"txt"`).
    /// When `None`, the format is auto-detected from the file extension.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

/// Tool that parses files and extracts their text content.
///
/// Supports plain text, CSV, and JSON formats. The format can be
/// auto-detected from the file extension or explicitly specified.
pub struct ParseFileTool;

impl ParseFileTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ParseFileTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TypedTool for ParseFileTool {
    type Args = ParseFileArgs;

    fn tool_id(&self) -> &str {
        "parse_file"
    }

    fn name(&self) -> &str {
        "parse_file"
    }

    fn description(&self) -> &str {
        "Parse a file and extract its text content. Supports plain text, CSV, and JSON formats."
    }

    fn category(&self) -> Option<&str> {
        Some("file-processing")
    }

    async fn execute(
        &self,
        args: Self::Args,
        _ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let parser = FileParser::new();

        let result = match args.format.as_deref() {
            Some("csv") => {
                let content = parser
                    .parse_text(&args.file_path)
                    .map_err(|e| ToolError::ExecutionFailed(e))?;
                let rows = parser
                    .parse_csv(&content)
                    .map_err(|e| ToolError::ExecutionFailed(e))?;
                serde_json::to_string_pretty(&rows)
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to serialize CSV: {e}")))
            }
            Some("json") => {
                let content = parser
                    .parse_text(&args.file_path)
                    .map_err(|e| ToolError::ExecutionFailed(e))?;
                parser
                    .parse_json(&content)
                    .map_err(|e| ToolError::ExecutionFailed(e))
            }
            Some(other) => parser
                .parse_text(&args.file_path)
                .map_err(|e| ToolError::ExecutionFailed(format!("Unsupported format '{other}': {e}"))),
            None => parser
                .parse_auto(&args.file_path)
                .map_err(|e| ToolError::ExecutionFailed(e)),
        };

        let content = result?;

        let data = serde_json::json!({
            "file_path": args.file_path,
            "format": args.format.unwrap_or_else(|| parser.detect_format(&args.file_path).to_string()),
            "content": content,
        });

        Ok(ToolResult::success("parse_file", data).into())
    }
}

// ---------------------------------------------------------------------------
// DescribeImageTool
// ---------------------------------------------------------------------------

/// Arguments for [`DescribeImageTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DescribeImageArgs {
    /// Image source: a URL (`http://` / `https://`) or base64-encoded data
    /// (raw base64 string or `data:` URI).
    pub source: String,
    /// Optional hint about what to look for in the image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description_hint: Option<String>,
}

/// Tool that describes a single image using the configured vision API provider.
///
/// The provider (OpenAI, Anthropic, or Ollama) is selected via the
/// `[multimodal]` config section's `vision_provider` field.
pub struct DescribeImageTool;

impl DescribeImageTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DescribeImageTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TypedTool for DescribeImageTool {
    type Args = DescribeImageArgs;

    fn tool_id(&self) -> &str {
        "describe_image"
    }

    fn name(&self) -> &str {
        "describe_image"
    }

    fn description(&self) -> &str {
        "Describe an image from a URL or base64-encoded source using the configured vision API provider (OpenAI, Anthropic, or Ollama)."
    }

    fn category(&self) -> Option<&str> {
        Some("multimodal")
    }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        // Load the multimodal config from the agent spec
        let config = ctx
            .agent_spec
            .config::<MultimodalConfigKey>()
            .map_err(|e| {
                ToolError::ExecutionFailed(format!("Failed to load multimodal config: {e}"))
            })?;

        // Create the vision client; fail early if vision is not configured
        let client = VisionClient::from_config(&config).ok_or_else(|| {
            ToolError::ExecutionFailed(
                "Vision is not configured. Set vision_provider in the [multimodal] config section "
                    .to_string(),
            )
        })?;

        // Call the vision API
        let description = client
            .describe_image(&args.source, args.description_hint.as_deref())
            .await
            .map_err(|e| ToolError::ExecutionFailed(e))?;

        let data = serde_json::json!({
            "source": args.source,
            "hint": args.description_hint,
            "description": description,
            "status": "success",
        });

        Ok(ToolResult::success("describe_image", data).into())
    }
}

// ---------------------------------------------------------------------------
// AnalyzeImageTool
// ---------------------------------------------------------------------------

/// Arguments for [`AnalyzeImageTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AnalyzeImageArgs {
    /// List of image sources (URLs or base64-encoded data) to analyze.
    pub sources: Vec<String>,
    /// Optional hint about what to look for in the images. Applied to all images.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description_hint: Option<String>,
}

/// Tool that analyzes multiple images and returns descriptions for each.
///
/// Uses the same vision provider as [`DescribeImageTool`] but processes
/// multiple images in sequence, returning a summary with success/failure
/// counts.
pub struct AnalyzeImageTool;

impl AnalyzeImageTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AnalyzeImageTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TypedTool for AnalyzeImageTool {
    type Args = AnalyzeImageArgs;

    fn tool_id(&self) -> &str {
        "analyze_images"
    }

    fn name(&self) -> &str {
        "analyze_images"
    }

    fn description(&self) -> &str {
        "Analyze multiple images and return descriptions for each. Accepts URLs or base64-encoded image data."
    }

    fn category(&self) -> Option<&str> {
        Some("multimodal")
    }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        if args.sources.is_empty() {
            return Err(ToolError::ExecutionFailed(
                "At least one image source is required".to_string(),
            ));
        }

        // Load the multimodal config from the agent spec
        let config = ctx
            .agent_spec
            .config::<MultimodalConfigKey>()
            .map_err(|e| {
                ToolError::ExecutionFailed(format!("Failed to load multimodal config: {e}"))
            })?;

        // Create the vision client
        let client = VisionClient::from_config(&config).ok_or_else(|| {
            ToolError::ExecutionFailed(
                "Vision is not configured. Set vision_provider in the [multimodal] config section."
                    .to_string(),
            )
        })?;

        // Process each image
        let mut descriptions = Vec::with_capacity(args.sources.len());
        for source in &args.sources {
            let entry = match client
                .describe_image(source, args.description_hint.as_deref())
                .await
            {
                Ok(desc) => serde_json::json!({
                    "source": source,
                    "description": desc,
                }),
                Err(e) => serde_json::json!({
                    "source": source,
                    "error": e,
                }),
            };
            descriptions.push(entry);
        }

        let total = descriptions.len();
        let successful = descriptions.iter().filter(|d| d.get("error").is_none()).count();

        let data = serde_json::json!({
            "descriptions": descriptions,
            "total": total,
            "successful": successful,
            "failed": total - successful,
        });

        Ok(ToolResult::success("analyze_images", data).into())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── ParseFileTool ────────────────────────────────────────────────────

    #[test]
    fn parse_file_tool_descriptor() {
        let tool = ParseFileTool::new();
        assert_eq!(tool.tool_id(), "parse_file");
        assert_eq!(tool.name(), "parse_file");
        assert!(tool.description().contains("Parse a file"));
        assert_eq!(tool.category(), Some("file-processing"));
    }

    #[test]
    fn parse_file_args_defaults() {
        let args = ParseFileArgs {
            file_path: "test.txt".to_string(),
            format: None,
        };
        assert!(args.format.is_none());
    }

    // ── DescribeImageTool ────────────────────────────────────────────────

    #[test]
    fn describe_image_tool_descriptor() {
        let tool = DescribeImageTool::new();
        assert_eq!(tool.tool_id(), "describe_image");
        assert_eq!(tool.name(), "describe_image");
        assert!(tool.description().contains("Describe an image"));
        assert_eq!(tool.category(), Some("multimodal"));
    }

    #[test]
    fn describe_image_args_defaults() {
        let args = DescribeImageArgs {
            source: "https://example.com/img.png".to_string(),
            description_hint: None,
        };
        assert!(args.description_hint.is_none());
    }

    #[test]
    fn describe_image_args_with_hint() {
        let args = DescribeImageArgs {
            source: "data:image/png;base64,iVBOR".to_string(),
            description_hint: Some("what color is the sky?".to_string()),
        };
        assert_eq!(
            args.description_hint.as_deref(),
            Some("what color is the sky?")
        );
    }

    // ── AnalyzeImageTool ─────────────────────────────────────────────────

    #[test]
    fn analyze_image_tool_descriptor() {
        let tool = AnalyzeImageTool::new();
        assert_eq!(tool.tool_id(), "analyze_images");
        assert_eq!(tool.name(), "analyze_images");
        assert!(tool.description().contains("Analyze multiple images"));
        assert_eq!(tool.category(), Some("multimodal"));
    }

    #[test]
    fn analyze_image_args_defaults() {
        let args = AnalyzeImageArgs {
            sources: vec!["https://example.com/img1.png".to_string()],
            description_hint: None,
        };
        assert_eq!(args.sources.len(), 1);
        assert!(args.description_hint.is_none());
    }

    #[test]
    fn analyze_image_args_empty_sources() {
        let args = AnalyzeImageArgs {
            sources: vec![],
            description_hint: None,
        };
        assert!(args.sources.is_empty());
    }

    #[test]
    fn analyze_image_args_multiple_sources() {
        let args = AnalyzeImageArgs {
            sources: vec![
                "https://example.com/img1.png".to_string(),
                "data:image/png;base64,iVBOR".to_string(),
            ],
            description_hint: Some("find all text".to_string()),
        };
        assert_eq!(args.sources.len(), 2);
        assert_eq!(args.description_hint.as_deref(), Some("find all text"));
    }
}
