use remo_runtime_contract::PluginConfigKey;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// SandboxConfigKey — typed plugin config key
// ---------------------------------------------------------------------------

pub struct SandboxConfigKey;

impl PluginConfigKey for SandboxConfigKey {
    const KEY: &'static str = "sandbox";
    type Config = SandboxConfig;
}

// ---------------------------------------------------------------------------
// SandboxProviderType — which sandbox backend to use
// ---------------------------------------------------------------------------

/// Sandbox backend implementation type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SandboxProviderType {
    /// Process-level isolation via OS sandboxing and resource limits.
    #[default]
    Process,
    /// Container-based isolation via Docker.
    Docker,
    /// WebAssembly-based isolation.
    Wasm,
}

// ---------------------------------------------------------------------------
// NetworkPolicyType — network access level inside the sandbox
// ---------------------------------------------------------------------------

/// Network access policy for sandboxed execution.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NetworkPolicyType {
    /// No network access inside the sandbox.
    Disabled,
    /// Outbound connections allowed (default).
    #[default]
    OutboundOnly,
    /// Full network access (inbound + outbound).
    Full,
}

// ---------------------------------------------------------------------------
// SandboxConfig — top-level sandbox plugin configuration
// ---------------------------------------------------------------------------

/// Configuration for the sandbox plugin, stored in `AgentSpec.sections["sandbox"]`.
///
/// All fields have sensible defaults so a minimal config `{}` is valid.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SandboxConfig {
    /// Which sandbox backend to use.
    #[serde(default)]
    pub provider: SandboxProviderType,

    /// Maximum memory the sandboxed process may consume (in megabytes).
    #[serde(default = "default_max_memory_mb")]
    pub max_memory_mb: u64,

    /// Maximum CPU time allowed before the process is killed (in seconds).
    #[serde(default = "default_max_cpu_time_s")]
    pub max_cpu_time_s: u64,

    /// Network access policy inside the sandbox.
    #[serde(default)]
    pub network_policy: NetworkPolicyType,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            provider: SandboxProviderType::default(),
            max_memory_mb: default_max_memory_mb(),
            max_cpu_time_s: default_max_cpu_time_s(),
            network_policy: NetworkPolicyType::default(),
        }
    }
}

fn default_max_memory_mb() -> u64 {
    256
}

fn default_max_cpu_time_s() -> u64 {
    30
}
