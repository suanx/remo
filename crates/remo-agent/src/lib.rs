//! Compatibility package that re-exports the primary `remo` umbrella crate.
//!
//! New code should depend on the published `remo` package directly. This
//! package exists so existing `remo-agent` users can move to the new release
//! line without changing import paths.

pub use remo_core::*;
