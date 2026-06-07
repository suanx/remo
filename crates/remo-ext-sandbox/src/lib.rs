//! # remo-ext-sandbox
//!
//! Process-level sandbox plugin for secure tool execution in the Remo AI Agent framework.
//!
//! This crate provides:
//!
//! - **`SandboxConfig`** – typed configuration stored in `AgentSpec.sections["sandbox"]`
//! - **`SandboxPolicy`** – resolved, runtime-ready policy derived from config
//! - **`ProcessSandbox`** – process-level sandbox provider using `tokio::process`
//! - **`SandboxToolGateHook`** – tool gate hook that validates and intercepts high-risk tool calls
//! - **`SandboxPlugin`** – plugin entry point that wires everything together
//!
//! ## Plugin Registration
//!
//! ```ignore
//! // In your agent builder:
//! registrar.register_plugin(SandboxPlugin);
//! ```
//!
//! ## Configuration
//!
//! Add a `sandbox` section to your agent spec:
//!
//! ```json
//! {
//!   "sandbox": {
//!     "provider": "process",
//!     "max_memory_mb": 512,
//!     "max_cpu_time_s": 60,
//!     "network_policy": "disabled"
//!   }
//! }
//! ```

pub mod config;
pub mod hook;
pub mod plugin;
pub mod policy;
pub mod process_sandbox;
pub mod provider;

// Re-exports for convenience
pub use config::{NetworkPolicyType, SandboxConfig, SandboxConfigKey, SandboxProviderType};
pub use hook::SandboxToolGateHook;
pub use plugin::{SandboxPlugin, SandboxStateKey, SANDBOX_PLUGIN_NAME};
pub use policy::{FileSystemPolicy, NetworkPolicy, ResourceLimits, SandboxPolicy};
pub use process_sandbox::ProcessSandbox;
pub use provider::{SandboxError, SandboxProvider, SandboxResult};
