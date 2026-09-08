//! This module owns lowercase hexadecimal rendering of 32-byte digests for pool names.

use std::fmt;

/// Renders one 32-byte digest as 64 lowercase hexadecimal characters.
///
/// Every immutable-pool and namespace filename in versions 1 and 2 derives its
/// digest component from this one renderer, so the on-disk grammar the
/// admission checks expect is emitted from a single place.
pub(super) struct DigestHex<'digest>(pub(super) &'digest [u8; 32]);

impl fmt::Display for DigestHex<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}
