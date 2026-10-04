//! This executable owns the process-isolated durable locator laws.
//!
//! Child creation must not share a process with golden-store writer handoffs:
//! a child can inherit a live flock description until exec. Separate test
//! executables have separate descriptor tables even when Cargo runs in parallel.
//! The locator parent owns no store; each admitted child runs one law serially.

#![cfg(target_os = "linux")]

#[path = "golden_file_worldline/durable_fixture.rs"]
#[allow(
    dead_code,
    reason = "locator laws share complete-store setup; other golden laws own partial-record cases"
)]
mod durable_fixture;
#[path = "golden_file_worldline/durable_locator_laws.rs"]
mod durable_locator_laws;
#[path = "segment_filesystem_stage/sandbox.rs"]
mod durable_sandbox;
