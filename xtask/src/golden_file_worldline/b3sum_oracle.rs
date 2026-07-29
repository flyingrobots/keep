//! This module owns the independent digest adapter for identity evidence.

use crate::external_digest;

use super::GoldenError;
use super::digest_port::IdentityDigestOracle;

const ALGORITHM: u8 = 1;
const DATA_MAGIC: [u8; 16] = *b"KEEP:BLOB:DATA\0\0";
const VERSION: u16 = 1;

pub(super) struct B3sumOracle;

impl IdentityDigestOracle for B3sumOracle {
    fn identity_digest(&self, payload: &[u8]) -> Result<[u8; 32], GoldenError> {
        let length = u64::try_from(payload.len()).map_err(|source| {
            GoldenError::violation(format!("payload length cannot be represented: {source}"))
        })?;
        let version = VERSION.to_be_bytes();
        let algorithm = [ALGORITHM];
        let length = length.to_be_bytes();
        external_digest::b3sum(&[
            DATA_MAGIC.as_slice(),
            version.as_slice(),
            algorithm.as_slice(),
            payload,
            length.as_slice(),
        ])
        .map_err(GoldenError::external_digest)
    }
}

#[cfg(test)]
mod tests {
    use super::{B3sumOracle, IdentityDigestOracle};
    use crate::golden_file_worldline::identity_oracle::digest;

    #[test]
    fn external_oracle_agrees_on_the_identity_preimage() {
        let payload = b"independent digest boundary";
        let external = B3sumOracle.identity_digest(payload);
        let internal = digest(payload);
        assert!(matches!(
            (external, internal),
            (Ok(external), Ok(internal)) if external == *internal.as_bytes()
        ));
    }
}
