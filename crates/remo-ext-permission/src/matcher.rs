//! Deprecated re-export shim for [`remo_tool_pattern`].
//!
//! The matching logic now lives in `remo-tool-pattern`. This module
//! re-exports the same symbols so external callers still compile; new
//! code should import from `remo_tool_pattern` directly.
#![allow(deprecated)]

#[deprecated(
    since = "0.5.1",
    note = "import from `remo_tool_pattern` directly; this shim will be removed in a future major release"
)]
pub use remo_tool_pattern::{
    MatchResult, Specificity, evaluate_field_condition, evaluate_op, op_precision, pattern_matches,
    resolve_path, schema_has_path, validate_pattern_fields, wildcard_match,
};
