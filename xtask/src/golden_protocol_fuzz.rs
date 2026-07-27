//! This module owns the dependency-free production parser fuzz facade.

#[allow(dead_code, reason = "the fuzz facade uses only pure parser paths")]
#[path = "golden_file_worldline/canonical_value.rs"]
mod canonical_value;
#[allow(dead_code, reason = "the fuzz facade performs no filesystem I/O")]
#[path = "golden_file_worldline/corpus_protocol.rs"]
mod corpus_protocol;
#[allow(
    dead_code,
    reason = "the fuzz facade retains the checker's error shape"
)]
#[path = "golden_file_worldline/error.rs"]
mod error;
#[path = "golden_file_worldline/fuzz_admission.rs"]
mod fuzz_admission;
#[allow(
    dead_code,
    reason = "the fuzz facade uses only identity classification"
)]
#[path = "golden_file_worldline/invalid_text_oracle.rs"]
mod invalid_text_oracle;
#[path = "golden_file_worldline/mutation_value.rs"]
mod mutation_value;

use corpus_protocol::Corpus;
pub(super) use error::GoldenError;

pub(super) fn admit(selector: u8, input: &[u8]) -> Result<(), GoldenError> {
    fuzz_admission::admit(selector, input)
}
