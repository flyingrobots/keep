//! This module owns the writer suite's fixture imports.

#[path = "../golden_file_worldline/durable_fixture.rs"]
#[allow(
    dead_code,
    reason = "writer laws use complete stores; golden laws cover partial-record construction"
)]
mod durable_fixture;
#[path = "../segment_filesystem_stage/sandbox.rs"]
#[allow(
    dead_code,
    reason = "writer laws use Drop cleanup; other filesystem laws check explicit removal"
)]
mod durable_sandbox;
#[path = "finalization_laws.rs"]
mod finalization_laws;
#[path = "format_marker_laws.rs"]
mod format_marker_laws;
#[path = "forward_laws.rs"]
mod forward_laws;
#[path = "../support/populated_store.rs"]
#[allow(
    dead_code,
    reason = "each consumer uses a subset of the shared builder; none reads every receipt"
)]
mod populated_store;
#[path = "recovery_fixture.rs"]
mod recovery_fixture;
#[path = "refusal_laws.rs"]
mod refusal_laws;
#[path = "stage_recovery_laws.rs"]
mod stage_recovery_laws;
#[path = "../support/successor_catalog.rs"]
mod successor_catalog;
#[path = "writer_fixture.rs"]
mod writer_fixture;
