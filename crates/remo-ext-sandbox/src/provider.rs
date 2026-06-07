use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::policy::SandboxPolicy;

// ---------------------------------------------------------------------------
// SandboxProvider trait — abstraction over different isolation backends
// ---------------------------------------------------------------------------

/// Backend abstraction for sandboxed command execution.
///
/// Implementations provide concrete isolation (process, Docker, Wasm, etc.)
/// while sharing a common policy-driven interface.
#[async_trait]
pub trait SandboxProvider: Send + Sync + 'static {
    /// Human-readable backend name (e.g. "process", "docker", "wasm").
    fn name(&self) -> &str;

    /// Execute a command string inside the sandbox, applying the given policy.
    async fn execute(
        &self,
        command: &str,
        policy: &SandboxPolicy,
    ) -> Result<SandboxResult, SandboxError>;
}

// ---------------------------------------------------------------------------
// SandboxResult — outcome of a sandboxed execution
// ---------------------------------------------------------------------------

/// The captured outcome of a sandboxed process execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxResult {
    /// Process exit code (0 = success).
    pub exit_code: i32,
    /// Captured stdout (may be truncated to `max_output_size`).
    pub stdout: String,
    /// Captured stderr (may be truncated to `max_output_size`).
    pub stderr: String,
    /// Wall-clock duration of the execution in milliseconds.
    pub duration_ms: u64,
}

// ---------------------------------------------------------------------------
// SandboxError — failure modes for sandboxed execution
// ---------------------------------------------------------------------------

/// Errors that can occur during sandboxed execution.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SandboxError {
    /// The process exceeded the allowed wall-clock time limit.
    #[error("sandbox execution timed out after {timeout_s}s")]
    Timeout { timeout_s: u64 },

    /// The process exceeded a resource limit (memory, output size, etc.).
    #[error("resource limit exceeded: {0}")]
    ResourceLimit(String),

    /// The process exited with an error or could not be spawned.
    #[error("execution error: {0}")]
    ExecutionError(String),

    /// The requested operation is not supported by this provider.
    #[error("unsupported operation: {0}")]
    UnsupportedOperation(String),
}
