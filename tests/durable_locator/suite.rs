//! This module owns the isolated locator suite's fixture imports.

#[path = "../golden_file_worldline/durable_fixture.rs"]
#[allow(
    dead_code,
    reason = "locator laws use complete stores; golden laws cover partial-record construction"
)]
mod durable_fixture;
#[path = "../golden_file_worldline/durable_locator_laws.rs"]
mod durable_locator_laws;
#[path = "../segment_filesystem_stage/sandbox.rs"]
#[allow(
    dead_code,
    reason = "locator laws use Drop cleanup; other filesystem laws check explicit removal"
)]
mod durable_sandbox;
