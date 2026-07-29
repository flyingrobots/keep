//! This module owns the conformance port to the shared external digest witness.

use crate::external_digest;

use super::ConformanceError;

const DIGEST_BYTES: usize = 32;

pub(super) fn digest(parts: &[&[u8]]) -> Result<[u8; DIGEST_BYTES], ConformanceError> {
    external_digest::b3sum(parts).map_err(ConformanceError::external_digest)
}
