//! Runtime checkpoint read port.
//!
//! `RuntimeCheckpointStore` is the narrow read port the runtime consumes; it
//! lives in `remo-runtime-contract`. The runtime library references nothing
//! from `remo-server-contract` (that would be a reverse dependency). The
//! `ThreadRunStore`-backed adapter (`ThreadRunCheckpointStore`) is a
//! server/store concern in server-contract; runtime tests that wire a
//! store-backed reader import it directly through the dev-dependency.

pub use remo_runtime_contract::contract::storage::RuntimeCheckpointStore;
