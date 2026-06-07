use std::time::Instant;

use async_trait::async_trait;
use tokio::process::Command;

use crate::policy::SandboxPolicy;
use crate::provider::{SandboxError, SandboxProvider, SandboxResult};

// ---------------------------------------------------------------------------
// ProcessSandbox — process-level sandbox provider
// ---------------------------------------------------------------------------

/// Process-level sandbox that executes commands as child processes with
/// resource limits and policy validation.
///
/// This is the default provider. It provides a working foundation using
/// OS-level process isolation — spawning commands via `tokio::process::Command`,
/// enforcing timeouts, and capturing output.
pub struct ProcessSandbox;

impl ProcessSandbox {
    /// Create a new process-level sandbox.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ProcessSandbox {
    fn default() -> Self {
        Self::new()
    }
}

/// Patterns that are considered high-risk and blocked by the sandbox.
const BLOCKED_PATTERNS: &[&str] = &[
    "rm -rf /",
    "mkfs",
    "> /dev/sd",
    "dd if=",
    ":(){ :|:& };:",
    "chmod -R 777 /",
    "wget",
    "curl",
    "nc -l",
    "ncat",
    "python -c",
    "perl -e",
];

/// Check whether a command string contains dangerous operations that should
/// be blocked regardless of policy.
fn is_dangerous_command(command: &str) -> Option<&'static str> {
    let lower = command.to_lowercase();
    for pattern in BLOCKED_PATTERNS {
        if lower.contains(pattern) {
            return Some(pattern);
        }
    }
    None
}

#[async_trait]
impl SandboxProvider for ProcessSandbox {
    fn name(&self) -> &str {
        "process"
    }

    async fn execute(
        &self,
        command: &str,
        policy: &SandboxPolicy,
    ) -> Result<SandboxResult, SandboxError> {
        // ── 1. Validate command against policy ──────────────────────────
        if let Some(blocked) = is_dangerous_command(command) {
            return Err(SandboxError::ExecutionError(format!(
                "command blocked by sandbox policy: contains dangerous pattern '{blocked}'"
            )));
        }

        // Validate network policy: block network tools when network is disabled
        if !policy.network_policy.outbound_allowed {
            let lower = command.to_lowercase();
            let network_cmds = ["curl", "wget", "ssh", "scp", "rsync", "ping", "nc", "ncat"];
            for cmd in &network_cmds {
                if lower.starts_with(cmd) || lower.contains(&format!(" {cmd} ")) {
                    return Err(SandboxError::ResourceLimit(format!(
                        "network command '{cmd}' blocked by policy: network access is disabled"
                    )));
                }
            }
        }

        // Validate file-system policy: block access to blocked paths
        for blocked_path in &policy.file_system.blocked_paths {
            if command.contains(blocked_path) {
                return Err(SandboxError::ResourceLimit(format!(
                    "command accesses blocked path: '{blocked_path}'"
                )));
            }
        }

        // ── 2. Prepare execution ────────────────────────────────────────
        let timeout_s = policy.resource_limits.max_cpu_time_s;
        let max_output = policy.resource_limits.max_output_size;

        // Split command for shell execution
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command);

        // Clear the environment to prevent leaking host env vars into sandbox
        cmd.env_clear();

        // Set a minimal, safe environment
        cmd.env("PATH", "/usr/local/bin:/usr/bin:/bin");
        cmd.env("HOME", "/tmp");
        cmd.env("LANG", "en_US.UTF-8");

        // ── 3. Spawn and wait with timeout ──────────────────────────────
        let start = Instant::now();

        let output = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_s),
            cmd.output(),
        )
        .await
        .map_err(|_| SandboxError::Timeout { timeout_s })?
        .map_err(|e| SandboxError::ExecutionError(format!("failed to spawn process: {e}")))?;

        let elapsed_ms = start.elapsed().as_millis() as u64;

        // ── 4. Capture and truncate output ──────────────────────────────
        let mut stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let mut stderr = String::from_utf8_lossy(&output.stderr).into_owned();

        // Truncate combined output to max_output_size
        let total_len = stdout.len() + stderr.len();
        if total_len > max_output {
            let budget_per_stream = max_output / 2;
            if stdout.len() > budget_per_stream {
                stdout.truncate(budget_per_stream);
                stdout.push_str("\n... [truncated]");
            }
            let remaining = max_output.saturating_sub(stdout.len());
            if stderr.len() > remaining {
                stderr.truncate(remaining);
                stderr.push_str("\n... [truncated]");
            }
        }

        let exit_code = output.status.code().unwrap_or(-1);

        Ok(SandboxResult {
            exit_code,
            stdout,
            stderr,
            duration_ms: elapsed_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{FileSystemPolicy, NetworkPolicy, ResourceLimits};

    fn test_policy() -> SandboxPolicy {
        SandboxPolicy {
            resource_limits: ResourceLimits {
                max_memory_mb: 128,
                max_cpu_time_s: 5,
                max_output_size: 1024,
            },
            network_policy: NetworkPolicy {
                outbound_allowed: false,
                allowed_hosts: Vec::new(),
            },
            file_system: FileSystemPolicy::default(),
        }
    }

    #[tokio::test]
    async fn basic_command_succeeds() {
        let sandbox = ProcessSandbox::new();
        let policy = test_policy();
        let result = sandbox.execute("echo hello", &policy).await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello"));
    }

    #[tokio::test]
    async fn dangerous_command_is_blocked() {
        let sandbox = ProcessSandbox::new();
        let policy = test_policy();
        let err = sandbox.execute("rm -rf /", &policy).await.unwrap_err();
        assert!(matches!(err, SandboxError::ExecutionError(_)));
    }

    #[tokio::test]
    async fn network_command_blocked_when_disabled() {
        let sandbox = ProcessSandbox::new();
        let policy = test_policy();
        let err = sandbox.execute("curl https://example.com", &policy).await.unwrap_err();
        assert!(matches!(err, SandboxError::ResourceLimit(_)));
    }
}
