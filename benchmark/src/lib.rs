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
mod measurement;
#[allow(
    clippy::redundant_pub_crate,
    reason = "measurement sequencing delegates checked aggregation here"
)]
mod measurement_aggregate;
mod measurement_error;
mod measurement_metrics;
#[allow(
    clippy::redundant_pub_crate,
    reason = "aggregation and its law tests share exact percentile selection"
)]
mod measurement_percentile;
#[allow(
    clippy::redundant_pub_crate,
    reason = "the public measurement facade delegates private execution here"
)]
mod measurement_run;
#[allow(
    clippy::redundant_pub_crate,
    reason = "the private aggregator consumes isolated sample evidence"
)]
mod measurement_sample;
#[allow(
    clippy::redundant_pub_crate,
    reason = "sibling scenario adapters share one private input meter"
)]
mod metered_reader;
mod profile;
mod profile_error;
mod profile_partition;
mod report;
mod report_environment;
mod report_error;
#[allow(
    clippy::redundant_pub_crate,
    reason = "report collection and serialization share profile evidence"
)]
mod report_profile;
#[allow(
    clippy::redundant_pub_crate,
    reason = "profile sequencing delegates checked aggregation here"
)]
mod report_profile_aggregate;
#[allow(
    clippy::redundant_pub_crate,
    reason = "the public report facade delegates canonical serialization here"
)]
mod report_tsv;
#[allow(
    clippy::redundant_pub_crate,
    reason = "the TSV sequencer delegates metadata rows here"
)]
mod report_tsv_metadata;
#[allow(
    clippy::redundant_pub_crate,
    reason = "the TSV sequencer delegates profile rows here"
)]
mod report_tsv_profile;
#[allow(
    clippy::redundant_pub_crate,
    reason = "the TSV sequencer delegates scenario rows here"
)]
mod report_tsv_scenario;
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
pub use measurement::{BaselineMeasurements, MeasurementSettings, ScenarioMetrics};
pub use measurement_error::MeasurementError;
pub use profile::{ChunkPartition, ChunkingProfile};
pub use profile_error::ProfileError;
pub use report::{BaselineReport, BuildProfile};
pub use report_environment::{BaselineEnvironment, HostDescription, SourceTreeState};
pub use report_error::ReportError;
pub use scenario::{PreparedScenario, Scenario, ScenarioObservation, VerificationPosture};
pub use scenario_error::ScenarioError;
