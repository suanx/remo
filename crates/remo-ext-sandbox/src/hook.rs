use async_trait::async_trait;

use remo_runtime::{PhaseContext, ToolGateHook};
use remo_runtime_contract::StateError;
use remo_runtime_contract::contract::tool_intercept::ToolInterceptPayload;

use crate::policy::SandboxPolicy;
use crate::plugin::SandboxStateKey;

// ---------------------------------------------------------------------------
// SandboxToolGateHook — intercepts high-risk tool calls for sandboxed execution
// ---------------------------------------------------------------------------

/// Tool gate hook that wraps execution of high-risk tools inside the sandbox.
///
/// The hook inspects the incoming tool call and:
/// 1. For non-execution tools (search, read, etc.) → returns `None` (pass-through).
/// 2. For execution tools (bash, shell, run, etc.) → evaluates the command
///    against the sandbox policy and either allows it to proceed normally
///    or blocks it via a `ToolInterceptPayload::Block`.
///
/// In a full implementation the hook would intercept the tool call and execute
/// it inside the sandbox, returning a `SetResult` payload with the sandboxed
/// output. For now it focuses on policy validation and blocking.
pub struct SandboxToolGateHook {
    /// Fallback policy used when no runtime policy is found in state.
    default_policy: SandboxPolicy,
}

impl SandboxToolGateHook {
    /// Create a new hook with the given default policy.
    ///
    /// The hook will prefer the runtime policy from state (`SandboxStateKey`)
    /// when available, falling back to this default.
    pub fn new(default_policy: SandboxPolicy) -> Self {
        Self { default_policy }
    }
}

/// Tool name patterns that should be evaluated for sandbox interception.
const EXECUTION_TOOL_PATTERNS: &[&str] = &[
    "bash", "shell", "exec", "execute", "run", "command", "terminal", "sh",
];

/// Check if a tool name matches execution / shell patterns.
fn is_execution_tool(tool_name: &str) -> bool {
    let lower = tool_name.to_ascii_lowercase();
    EXECUTION_TOOL_PATTERNS
        .iter()
        .any(|pat| lower.contains(pat))
}

/// Extract the command string from tool arguments.
///
/// Supports common argument shapes:
/// - `{"command": "..."}`
/// - `{"cmd": "..."}`
/// - `{"script": "..."}`
/// - `{"code": "..."}`
fn extract_command(args: &serde_json::Value) -> Option<String> {
    let obj = args.as_object()?;
    for key in &["command", "cmd", "script", "code"] {
        if let Some(val) = obj.get(*key) {
            if let Some(s) = val.as_str() {
                return Some(s.to_string());
            }
        }
    }
    // Fallback: if the args are a single string, treat it as the command
    args.as_str().map(|s| s.to_string())
}

#[async_trait]
impl ToolGateHook for SandboxToolGateHook {
    async fn run(&self, ctx: &PhaseContext) -> Result<Option<ToolInterceptPayload>, StateError> {
        let tool_name = match &ctx.tool_name {
            Some(name) => name.as_str(),
            None => return Ok(None),
        };

        // Only intercept execution / shell tools
        if !is_execution_tool(tool_name) {
            return Ok(None);
        }

        let tool_args = ctx.tool_args.clone().unwrap_or_default();

        let command = match extract_command(&tool_args) {
            Some(cmd) => cmd,
            None => {
                // No command found in args — let the tool proceed (it might
                // not need sandboxing).
                return Ok(None);
            }
        };

        // Resolve effective policy: prefer runtime state, fall back to default
        let policy_ref = ctx.state::<SandboxStateKey>().unwrap_or(&self.default_policy);

        // ── Policy validation ──────────────────────────────────────────
        // Block dangerous commands regardless of provider
        if let Some(dangerous) = is_dangerous_command(&command) {
            return Ok(Some(ToolInterceptPayload::Block {
                reason: format!(
                    "Sandbox policy blocked execution of '{tool_name}': \
                     command contains dangerous pattern '{dangerous}'"
                ),
            }));
        }

        // Validate file-system policy
        for blocked_path in &policy_ref.file_system.blocked_paths {
            if command.contains(blocked_path) {
                return Ok(Some(ToolInterceptPayload::Block {
                    reason: format!(
                        "Sandbox policy blocked execution of '{tool_name}': \
                         command accesses blocked path '{blocked_path}'"
                    ),
                }));
            }
        }

        // Validate network policy
        if !policy_ref.network_policy.outbound_allowed {
            let lower = command.to_lowercase();
            let network_cmds = ["curl", "wget", "ssh", "scp", "rsync", "ping"];
            for cmd in &network_cmds {
                if lower.starts_with(cmd) || lower.contains(&format!(" {cmd} ")) {
                    return Ok(Some(ToolInterceptPayload::Block {
                        reason: format!(
                            "Sandbox policy blocked execution of '{tool_name}': \
                             network command '{cmd}' not allowed (network disabled)"
                        ),
                    }));
                }
            }
        }

        // Command passes policy checks — let it execute normally via the runtime.
        // In a future iteration this could execute inside a ProcessSandbox
        // and return a SetResult payload instead.
        Ok(None)
    }
}

/// Check whether a command string contains dangerous operations.
fn is_dangerous_command(command: &str) -> Option<&'static str> {
    const BLOCKED: &[&str] = &[
        "rm -rf /",
        "mkfs",
        "> /dev/sd",
        "dd if=",
        ":(){ :|:& };:",
        "chmod -R 777 /",
    ];
    let lower = command.to_lowercase();
    for pattern in BLOCKED {
        if lower.contains(pattern) {
            return Some(pattern);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{FileSystemPolicy, NetworkPolicy, ResourceLimits};

    fn test_policy() -> SandboxPolicy {
        SandboxPolicy {
            resource_limits: ResourceLimits::default(),
            network_policy: NetworkPolicy {
                outbound_allowed: false,
                allowed_hosts: Vec::new(),
            },
            file_system: FileSystemPolicy {
                read_only_paths: Vec::new(),
                blocked_paths: vec!["/etc/shadow".to_string()],
            },
        }
    }

    fn make_context(tool_name: &str, args: serde_json::Value) -> PhaseContext {
        let mut ctx = PhaseContext::new(
            remo_runtime_contract::model::Phase::BeforeToolExecute,
            Default::default(),
        );
        ctx.tool_name = Some(tool_name.to_string());
        ctx.tool_args = Some(args);
        ctx
    }

    #[tokio::test]
    async fn non_execution_tool_passes_through() {
        let hook = SandboxToolGateHook::new(test_policy());
        let ctx = make_context("read_file", serde_json::json!({"path": "/tmp/foo"}));
        let result = hook.run(&ctx).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn dangerous_command_is_blocked() {
        let hook = SandboxToolGateHook::new(test_policy());
        let ctx = make_context("bash", serde_json::json!({"command": "rm -rf /"}));
        let result = hook.run(&ctx).await.unwrap();
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), ToolInterceptPayload::Block { .. }));
    }

    #[tokio::test]
    async fn network_command_blocked_when_disabled() {
        let hook = SandboxToolGateHook::new(test_policy());
        let ctx = make_context("shell", serde_json::json!({"command": "curl https://example.com"}));
        let result = hook.run(&ctx).await.unwrap();
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), ToolInterceptPayload::Block { .. }));
    }

    #[tokio::test]
    async fn blocked_path_is_denied() {
        let hook = SandboxToolGateHook::new(test_policy());
        let ctx = make_context("bash", serde_json::json!({"command": "cat /etc/shadow"}));
        let result = hook.run(&ctx).await.unwrap();
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), ToolInterceptPayload::Block { .. }));
    }

    #[tokio::test]
    async fn safe_command_passes_through() {
        let hook = SandboxToolGateHook::new(test_policy());
        let ctx = make_context("bash", serde_json::json!({"command": "echo hello"}));
        let result = hook.run(&ctx).await.unwrap();
        assert!(result.is_none());
    }
}
