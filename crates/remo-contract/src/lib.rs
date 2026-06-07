//! Compatibility facade for Remo contract crates.
//!
//! ⚠️ **DEPRECATED** — This crate is kept for backward compatibility only.
//!
//! New runtime-facing code should depend on [`remo-runtime-contract`](https://crates.io/crates/remo-runtime-contract).
//! New server/store-facing code should depend on [`remo-server-contract`](https://crates.io/crates/remo-server-contract).
//!
//! This crate will be removed in a future major version. Please migrate your
//! imports to the direct sub-crate dependencies listed above.

#![allow(missing_docs)]
#![deprecated(
    since = "0.5.1-dev",
    note = "use `remo-runtime-contract` or `remo-server-contract` directly"
)]

pub use remo_runtime_contract::*;
pub use remo_server_contract as server_contract;
pub use remo_server_contract::contract;
pub use remo_server_contract::*;
