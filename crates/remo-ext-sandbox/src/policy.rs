use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::config::{NetworkPolicyType, SandboxConfig};

// ---------------------------------------------------------------------------
// SandboxPolicy — resolved, runtime-ready policy derived from SandboxConfig
// ---------------------------------------------------------------------------

/// Complete sandbox policy applied to every sandboxed execution.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SandboxPolicy {
    /// Resource consumption limits.
    pub resource_limits: ResourceLimits,
    /// Network access rules.
    pub network_policy: NetworkPolicy,
    /// File-system access rules.
    pub file_system: FileSystemPolicy,
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        Self {
            resource_limits: ResourceLimits::default(),
            network_policy: NetworkPolicy::default(),
            file_system: FileSystemPolicy::default(),
        }
    }
}

impl From<&SandboxConfig> for SandboxPolicy {
    fn from(config: &SandboxConfig) -> Self {
        Self {
            resource_limits: ResourceLimits {
                max_memory_mb: config.max_memory_mb,
                max_cpu_time_s: config.max_cpu_time_s,
                ..ResourceLimits::default()
            },
            network_policy: match config.network_policy {
                NetworkPolicyType::Disabled => NetworkPolicy {
                    outbound_allowed: false,
                    ..NetworkPolicy::default()
                },
                NetworkPolicyType::OutboundOnly => NetworkPolicy {
                    outbound_allowed: true,
                    ..NetworkPolicy::default()
                },
                NetworkPolicyType::Full => NetworkPolicy {
                    outbound_allowed: true,
                    allowed_hosts: vec!["*".to_string()],
                },
            },
            file_system: FileSystemPolicy::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// ResourceLimits
// ---------------------------------------------------------------------------

/// CPU, memory, and output constraints for a sandboxed process.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResourceLimits {
    /// Maximum memory in megabytes.
    pub max_memory_mb: u64,
    /// Maximum wall-clock CPU time in seconds.
    pub max_cpu_time_s: u64,
    /// Maximum output size in bytes (stdout + stderr combined).
    pub max_output_size: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 256,
            max_cpu_time_s: 30,
            max_output_size: 1024 * 1024, // 1 MiB
        }
    }
}

// ---------------------------------------------------------------------------
// NetworkPolicy
// ---------------------------------------------------------------------------

/// Controls which network connections a sandboxed process may open.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NetworkPolicy {
    /// Whether outbound connections are permitted.
    pub outbound_allowed: bool,
    /// Whitelist of allowed hostnames / IP patterns.
    /// An empty vec with `outbound_allowed = false` blocks everything.
    pub allowed_hosts: Vec<String>,
}

impl Default for NetworkPolicy {
    fn default() -> Self {
        Self {
            outbound_allowed: true,
            allowed_hosts: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// FileSystemPolicy
// ---------------------------------------------------------------------------

/// File-system visibility rules for a sandboxed process.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileSystemPolicy {
    /// Paths that are always read-only (writes silently ignored or denied).
    pub read_only_paths: Vec<String>,
    /// Paths that are completely blocked (access returns an error).
    pub blocked_paths: Vec<String>,
}

impl Default for FileSystemPolicy {
    fn default() -> Self {
        Self {
            read_only_paths: Vec::new(),
            blocked_paths: Vec::new(),
        }
    }
}
