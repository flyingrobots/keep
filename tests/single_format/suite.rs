//! This module owns the single-format suite's fixture imports.

#[cfg(feature = "repository-tasks")]
#[path = "acceptance_laws.rs"]
mod acceptance_laws;
#[path = "../golden_file_worldline/durable_fixture.rs"]
#[allow(
    dead_code,
    reason = "these laws use complete stores; golden laws cover partial-record construction"
)]
mod durable_fixture;
#[path = "../segment_filesystem_stage/sandbox.rs"]
#[allow(
    dead_code,
    reason = "these laws use Drop cleanup; other filesystem laws check explicit removal"
)]
mod durable_sandbox;
#[path = "initialization_laws.rs"]
mod initialization_laws;
#[path = "../support/populated_store.rs"]
#[allow(
    dead_code,
    reason = "each consumer uses a subset of the shared builder; none reads every receipt"
)]
mod populated_store;
#[path = "portability_laws.rs"]
mod portability_laws;
#[path = "../version_two_catalog_writer/recovery_fixture.rs"]
#[allow(dead_code, reason = "these laws use finalization only")]
mod recovery_fixture;
#[path = "store_copy.rs"]
mod store_copy;
#[path = "../support/successor_catalog.rs"]
#[allow(
    dead_code,
    reason = "these laws publish successors only through `publish`"
)]
mod successor_catalog;
#[path = "unsupported_directory.rs"]
mod unsupported_directory;
#[path = "../version_two_catalog_writer/writer_fixture.rs"]
#[allow(
    dead_code,
    unused_imports,
    reason = "these laws use a subset of the writer fixture and its re-exports"
)]
mod writer_fixture;
