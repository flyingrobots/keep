//! Reproducible performance evidence for Keep's streaming CAS.

#![deny(warnings)]
#![forbid(unsafe_code)]

mod corpus;
#[allow(
    clippy::redundant_pub_crate,
    reason = "sibling corpus modules share deterministic byte generators"
)]
mod corpus_bytes;
#[allow(
    clippy::redundant_pub_crate,
    reason = "corpus assembly delegates edit relationships to this module"
)]
mod corpus_edits;
mod corpus_error;
mod corpus_generation;
#[allow(
    clippy::redundant_pub_crate,
    reason = "corpus assembly delegates identity accounting to this module"
)]
mod corpus_identity;
#[allow(
    clippy::redundant_pub_crate,
    reason = "sibling scenario adapters share one private output counter"
)]
mod counting_writer;
mod fixed_profile;
mod git_cas_profile;
mod keep_profile;
#[allow(
    clippy::redundant_pub_crate,
    reason = "sibling scenario adapters share one private input meter"
)]
mod metered_reader;
mod profile;
mod profile_error;
mod profile_partition;
mod scenario;
mod scenario_error;
mod scenario_execution;
#[allow(
    clippy::redundant_pub_crate,
    reason = "the sibling prepared-state dispatcher owns scenario execution"
)]
mod scenario_ingest;
#[allow(
    clippy::redundant_pub_crate,
    reason = "ingest sequencing and read metrics share one operation boundary"
)]
mod scenario_ingest_operation;
#[allow(
    clippy::redundant_pub_crate,
    reason = "sibling scenario adapters share one private counter type"
)]
mod scenario_observation;
#[allow(
    clippy::redundant_pub_crate,
    reason = "range execution delegates exact accounting to this module"
)]
mod scenario_range_metrics;
#[allow(
    clippy::redundant_pub_crate,
    reason = "prepared scenarios delegate request generation to this module"
)]
mod scenario_ranges;
#[allow(
    clippy::redundant_pub_crate,
    reason = "the sibling prepared-state dispatcher owns scenario execution"
)]
mod scenario_read;

pub use corpus::BenchmarkCorpus;
pub use corpus_error::CorpusError;
pub use profile::{ChunkPartition, ChunkingProfile};
pub use profile_error::ProfileError;
pub use scenario::{PreparedScenario, Scenario, ScenarioObservation, VerificationPosture};
pub use scenario_error::ScenarioError;
