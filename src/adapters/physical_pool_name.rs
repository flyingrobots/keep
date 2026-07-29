//! Exact immutable-pool filename emission.

use std::fmt;

use super::SegmentDigest;
use crate::{CatalogDigest, CatalogGeneration};

pub(super) fn segment(digest: SegmentDigest) -> String {
    format!("{}.seg", DigestHex(digest.as_bytes()))
}

pub(super) fn catalog(generation: CatalogGeneration, digest: CatalogDigest) -> String {
    format!(
        "{:016x}-{}.cat",
        generation.get(),
        DigestHex(digest.as_bytes())
    )
}

struct DigestHex<'a>(&'a [u8; 32]);

impl fmt::Display for DigestHex<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}
