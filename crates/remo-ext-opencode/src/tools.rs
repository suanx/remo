//! Tools for interacting with OpenCode CLI and Zen API.

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use remo_runtime_contract::contract::tool::{
    ToolCallContext, ToolError, ToolOutput, ToolResult, TypedTool,
};
use remo_runtime_contract::PluginConfigKey;

use crate::config::{OpenCodeConfig, OpenCodeConfigKey};
use crate::model_discovery;

// ---------------------------------------------------------------------------
// OpenCodeExecTool
// ---------------------------------------------------------------------------

/// Arguments for [`OpenCodeExecTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OpenCodeExecArgs {
    /// Task description for OpenCode CLI to execute.
    pub task: String,
    /// Working directory (default: current directory).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
    /// Model ID to use (optional). Uses OpenCode's default if not set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Timeout in seconds (default: 300).
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_timeout() -> u64 { 300 }

/// Tool that executes OpenCode CLI for code generation and project tasks.
pub struct OpenCodeExecTool;

#[async_trait]
impl TypedTool for OpenCodeExecTool {
    type Args = OpenCodeExecArgs;

    fn tool_id(&self) -> &str { "opencode:exec" }
    fn name(&self) -> &str { "OpenCode Execute" }
    fn description(&self) -> &str {
        "Execute OpenCode CLI with a task description. OpenCode is an open-source AI coding agent. \
         Use this for code generation, refactoring, testing, and project-level tasks. \
         Falls back to a simulated response if the CLI is not installed."
    }
    fn category(&self) -> Option<&str> { Some("opencode") }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let config: OpenCodeConfig = ctx
            .agent_spec
            .config::<OpenCodeConfigKey>()
            .map_err(|e| ToolError::Internal(format!("Failed to read config: {e}")))?;

        if !config.enable_cli_tool {
            return Ok(ToolResult::success("opencode:exec", json!({
                "status": "disabled",
                "message": "OpenCode CLI tool is disabled in configuration. Enable it in the opencode section.",
            })).into());
        }

        // Check if opencode CLI is available in PATH or configured path
        let cli_path = resolve_cli_path(&config);
        let cli_available = cli_path.as_ref().map(|p| std::path::Path::new(p).exists()).unwrap_or(false);

        if !cli_available {
            // CLI not available — return informative message
            return Ok(ToolResult::success("opencode:exec", json!({
                "status": "cli_not_installed",
                "message": "OpenCode CLI is not installed or not found in PATH. Install it with: npm i -g opencode-ai",
                "task": args.task,
                "hint": "Falling back to the agent's built-in capabilities for this task.",
            })).into());
        }

        let binary = cli_path.unwrap_or_else(|| "opencode".to_string());
        let timeout = std::time::Duration::from_secs(args.timeout_secs.min(config.cli_timeout_secs));

        // Build command args
        let mut cmd = tokio::process::Command::new(&binary);
        cmd.arg("--non-interactive");  // Non-interactive mode
        cmd.arg("--yes");              // Auto-confirm
        cmd.arg(&args.task);
        
        if let Some(ref dir) = args.working_dir {
            cmd.current_dir(dir);
        }
        if let Some(ref model) = args.model {
            cmd.env("OPENCODE_MODEL", model);
        }
        // Use the Zen API key if configured
        if let Some(ref key) = config.zen_api_key {
            cmd.env("OPENCODE_ZEN_API_KEY", key);
        }

        let output = tokio::time::timeout(timeout, cmd.output()).await
            .map_err(|_| ToolError::ExecutionFailed("OpenCode CLI timed out".into()))?
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to run OpenCode CLI: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            Ok(ToolResult::success("opencode:exec", json!({
                "status": "completed",
                "stdout": stdout,
                "task": args.task,
            })).into())
        } else {
            Ok(ToolResult::success("opencode:exec", json!({
                "status": "failed",
                "stdout": stdout,
                "stderr": stderr,
                "exit_code": output.status.code(),
                "task": args.task,
            })).into())
        }
    }
}

fn resolve_cli_path(config: &OpenCodeConfig) -> Option<String> {
    config.cli_binary_path.clone().or_else(|| {
        // Try to find opencode in PATH
        std::env::var_os("PATH")
            .and_then(|paths| {
                std::env::split_paths(&paths)
                    .find_map(|dir| {
                        let candidates = if cfg!(windows) {
                            vec![dir.join("opencode.cmd"), dir.join("opencode.exe"), dir.join("opencode")]
                        } else {
                            vec![dir.join("opencode")]
                        };
                        candidates.into_iter().find(|p| p.exists())
                    })
                    .map(|p| p.to_string_lossy().to_string())
            })
    })
}

// ---------------------------------------------------------------------------
// OpenCodeListModelsTool
// ---------------------------------------------------------------------------

/// Arguments for [`OpenCodeListModelsTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OpenCodeListModelsArgs {
    /// Whether to include paid models in the listing.
    #[serde(default)]
    pub include_paid: bool,
}

/// Tool that lists available models from OpenCode Zen.
pub struct OpenCodeListModelsTool;

#[async_trait]
impl TypedTool for OpenCodeListModelsTool {
    type Args = OpenCodeListModelsArgs;

    fn tool_id(&self) -> &str { "opencode:list_models" }
    fn name(&self) -> &str { "OpenCode List Models" }
    fn description(&self) -> &str {
        "List available models from OpenCode Zen, including free models."
    }
    fn category(&self) -> Option<&str> { Some("opencode") }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let config: OpenCodeConfig = ctx
            .agent_spec
            .config::<OpenCodeConfigKey>()
            .map_err(|e| ToolError::Internal(format!("Failed to read config: {e}")))?;

        let mut models = if config.auto_discover_free_models {
            model_discovery::discover_models(&config).await
        } else {
            crate::config::builtin_free_models()
        };

        if !args.include_paid {
            models.retain(|m| m.is_free);
        }

        Ok(ToolResult::success("opencode:list_models", json!({
            "models": models,
            "count": models.len(),
            "free_count": models.iter().filter(|m| m.is_free).count(),
        })).into())
    }
}

// ---------------------------------------------------------------------------
// OpenCodeCheckCliTool
// ---------------------------------------------------------------------------

/// Arguments for [`OpenCodeCheckCliTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OpenCodeCheckCliArgs;

/// Tool that checks if OpenCode CLI is installed and provides setup instructions.
pub struct OpenCodeCheckCliTool;

#[async_trait]
impl TypedTool for OpenCodeCheckCliTool {
    type Args = OpenCodeCheckCliArgs;

    fn tool_id(&self) -> &str { "opencode:check_cli" }
    fn name(&self) -> &str { "OpenCode Check CLI" }
    fn description(&self) -> &str {
        "Check if OpenCode CLI is installed and provide setup instructions."
    }
    fn category(&self) -> Option<&str> { Some("opencode") }

    async fn execute(
        &self,
        _args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let config: OpenCodeConfig = ctx
            .agent_spec
            .config::<OpenCodeConfigKey>()
            .map_err(|e| ToolError::Internal(format!("Failed to read config: {e}")))?;

        let cli_path = resolve_cli_path(&config);
        let installed = cli_path.as_ref().map(|p| std::path::Path::new(p).exists()).unwrap_or(false);
        let version = if installed {
            match std::process::Command::new(cli_path.as_deref().unwrap_or("opencode"))
                .arg("--version")
                .output()
            {
                Ok(out) => String::from_utf8_lossy(&out.stdout).trim().to_string(),
                Err(_) => "unknown".to_string(),
            }
        } else {
            String::new()
        };

        Ok(ToolResult::success("opencode:check_cli", json!({
            "installed": installed,
            "version": if version.is_empty() { serde_json::Value::Null } else { json!(version) },
            "path": cli_path,
            "auto_discover_free_models": config.auto_discover_free_models,
            "free_models": crate::config::builtin_free_models().iter().map(|m| json!({
                "id": m.id,
                "name": m.name,
            })).collect::<Vec<_>>(),
            "install_instructions": "npm i -g opencode-ai",
        })).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opencode_exec_tool_descriptor() {
        let tool = OpenCodeExecTool;
        assert_eq!(tool.tool_id(), "opencode:exec");
        assert_eq!(tool.category(), Some("opencode"));
    }

    #[test]
    fn opencode_list_models_tool_descriptor() {
        let tool = OpenCodeListModelsTool;
        assert_eq!(tool.tool_id(), "opencode:list_models");
    }

    #[test]
    fn opencode_check_cli_tool_descriptor() {
        let tool = OpenCodeCheckCliTool;
        assert_eq!(tool.tool_id(), "opencode:check_cli");
    }
}
