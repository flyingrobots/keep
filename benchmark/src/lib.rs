//! Reproducible performance evidence for Keep's streaming CAS.

#![deny(warnings)]
#![forbid(unsafe_code)]

mod corpus;
mod corpus_error;
mod corpus_generation;

pub use corpus::BenchmarkCorpus;
pub use corpus_error::CorpusError;
