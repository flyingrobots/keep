//! Golden File Worldline integration-test entry point.

pub(crate) mod support;

#[path = "golden_file_worldline/suite.rs"]
mod suite;

#[cfg(target_os = "linux")]
#[path = "layout_mutations/support.rs"]
pub(crate) mod layout_mutation_support;
