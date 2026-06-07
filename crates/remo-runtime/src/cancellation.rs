//! Re-export cancellation types from remo-contract.
//!
//! The canonical definition now lives in `remo_runtime_contract::cancellation`.
//! This module preserves `crate::cancellation::*` import paths within the runtime.

#[allow(unused_imports)]
pub use remo_runtime_contract::cancellation::{CancellationHandle, CancellationToken};
