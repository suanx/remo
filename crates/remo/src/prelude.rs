//! One-import convenience module for remo.
//!
//! ```rust,ignore
//! use remo::prelude::*;
//! ```
//!
//! Covers the types needed to build agents, define tools, manage state,
//! and wire up plugins. Extension types are included when the corresponding
//! feature flag is active.

// ── Building agents ──
pub use crate::{AgentRuntime, AgentRuntimeBuilder, BuildError, RunActivation, RuntimeError};
pub use crate::{
    AgentSpec, AgentSpecPatch, ConfigRecord, Modalities, Modality, ModelSpec, PluginConfigKey,
    PreparedSkillSpecs, SkillArgumentSpec, SkillSpec, SkillSpecContext, SkillSpecPatch,
    SkillSpecSink,
};
pub use crate::{
    decode_config_record, effective_config_record, validate_agent_spec, validate_agent_spec_patch,
    validate_config_record, validate_config_record_overrides, validate_model_spec,
    validate_provider_spec, validate_skill_spec,
};
pub use remo_runtime::engine::MockProviderProfile;

// ── Plugin system ──
pub use crate::CancellationToken;
pub use crate::{EffectSpec, ScheduledActionSpec, TypedEffect};
pub use crate::{Phase, PhaseContext, PhaseHook, ToolGateHook};
pub use crate::{Plugin, PluginDescriptor, PluginRegistrar};
pub use crate::{TypedEffectHandler, TypedScheduledActionHandler};

// ── State ──
pub use crate::state::{CommitEvent, CommitHook, MutationBatch, StateCommand, StateStore};
pub use crate::{KeyScope, MergeStrategy, StateKey, StateKeyOptions};
pub use crate::{Snapshot, StateError, StateMap};

// ── Tools ──
pub use remo_runtime_contract::contract::tool::{
    Tool, ToolCallContext, ToolDescriptor, ToolError, ToolOutput, ToolResult, ToolStatus,
    ToolValidationError, TypedTool,
};
pub use remo_runtime_contract::contract::tool_schema::{
    generate_tool_schema, sanitize_for_llm, validate_against_schema,
};

// ── Tool interception (ToolGate decisions) ──
pub use remo_runtime_contract::contract::tool_intercept::ToolInterceptPayload;

// ── Context messages (system reminders / prompt injection) ──
pub use remo_runtime_contract::contract::context_message::{
    ContextMessage, ContextMessageTarget,
};

// ── Messages & content ──
pub use remo_runtime_contract::contract::content::ContentBlock;
pub use remo_runtime_contract::contract::message::{Message, Role, Visibility};

// ── Inference ──
pub use remo_runtime_contract::contract::executor::{InferenceRequest, LlmExecutor};
pub use remo_runtime_contract::contract::inference::{
    InferenceOverride, StopReason, StreamResult,
};

// ── Events ──
pub use remo_runtime_contract::contract::event::AgentEvent;
pub use remo_runtime_contract::contract::event_sink::EventSink;

// ── Lifecycle ──
pub use remo_runtime_contract::contract::lifecycle::{RunStatus, TerminationReason};

// ── Stop policies ──
pub use crate::policies::{StopConditionPlugin, StopDecision, StopPolicy, StopPolicyStats};

// ── Common re-exports ──
pub use serde_json::Value as JsonValue;
pub use std::sync::Arc;

// ── Extension types (feature-gated) ──

#[cfg(feature = "permission")]
pub use remo_ext_permission::{
    PermissionConfigKey, PermissionPlugin, PermissionRuleEntry, PermissionRulesConfig,
    ToolPermissionBehavior,
};

#[cfg(feature = "observability")]
pub use remo_ext_observability::ObservabilityPlugin;

#[cfg(feature = "mcp")]
pub use remo_ext_mcp::{McpPlugin, McpServerConnectionConfig, McpToolRegistryManager};

#[cfg(feature = "skills")]
pub use remo_ext_skills::{
    ConfigSkill, ConfigSkillRegistry, SkillDiscoveryPlugin, SkillRegistry,
};

#[cfg(feature = "reminder")]
pub use remo_ext_reminder::{
    ReminderPlugin, ReminderRule, ReminderRuleEntry, ReminderRulesConfig,
};

#[cfg(feature = "generative-ui")]
pub use remo_ext_generative_ui::{
    A2uiPlugin, A2uiPromptConfig, A2uiPromptConfigKey, DEFAULT_A2UI_CATALOG_ID,
};

#[cfg(feature = "search")]
pub use remo_ext_search::{SearchConfig, SearchConfigKey, SearchPlugin, SearchProvider};

#[cfg(feature = "evaluator")]
pub use remo_ext_evaluator::{
    EvaluatorConfig, EvaluatorConfigKey, EvaluatorPlugin, EvaluationCriterion,
};

#[cfg(feature = "notifications")]
pub use remo_ext_notifications::{NotificationConfig, NotificationConfigKey, NotificationPlugin};

#[cfg(feature = "voice")]
pub use remo_ext_voice::{VoiceConfig, VoiceConfigKey, VoicePlugin};
