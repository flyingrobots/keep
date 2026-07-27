//! This module owns the semantic boundary for an independent identity digest.

use super::GoldenError;

pub(super) trait IdentityDigestOracle {
    fn identity_digest(&self, payload: &[u8]) -> Result<[u8; 32], GoldenError>;
}
