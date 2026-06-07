//! Forwarding sink: sub-agent TextDelta -> parent ToolCallStreamDelta.
//!
//! Re-exported from [`remo_runtime::child_agent::sink`] under the legacy
//! name `StreamingSubagentSink`. New code should prefer
//! [`remo_runtime::StreamingPassthroughSink`] directly.

pub use remo_runtime::child_agent::sink::StreamingPassthroughSink as StreamingSubagentSink;
