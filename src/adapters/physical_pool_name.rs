//! Exact immutable-pool filename emission.

use super::SegmentDigest;
use super::digest_hex::DigestHex;
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
