//! Reproducible performance evidence for Keep's streaming CAS.

#![deny(warnings)]
#![forbid(unsafe_code)]

mod corpus;
mod corpus_error;
mod corpus_generation;
mod fixed_profile;
mod git_cas_profile;
mod keep_profile;
mod profile;
mod profile_error;
mod profile_partition;

pub use corpus::BenchmarkCorpus;
pub use corpus_error::CorpusError;
pub use profile::{ChunkPartition, ChunkingProfile};
pub use profile_error::ProfileError;
