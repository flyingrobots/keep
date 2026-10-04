//! Shared admitted retention publication preparation fixtures.

use std::error::Error;

use keep::{
    AdmittedCatalog, AdmittedRetentionRoot, AdmittedSegment, CanonicalRetentionRoot,
    CatalogSnapshot, ChecksummedCatalog, ChecksummedPublicationHead, LayoutEntryLimit,
    RetentionNamespace, RetentionPolicy, RetentionRoot, SegmentReadPolicy, SegmentRecordLimit,
};

use crate::support::decode_hex;

/// Frozen canonical generation-one root.
pub const ROOT_HEX: &str = include_str!("../../conformance/segment-store/v2/one-anchor-root.hex");
/// Frozen canonical generation-one manifest.
pub const MANIFEST_HEX: &str =
    include_str!("../../conformance/segment-store/v2/one-root-manifest.hex");
/// Frozen canonical generation-one retention head.
pub const HEAD_HEX: &str = include_str!("../../conformance/segment-store/v2/one-root-head.hex");
const SEGMENT_HEX: &str =
    include_str!("../../conformance/segment-store/v1/one-zero-bundle-segment.hex");
const CATALOG_HEX: &str =
    include_str!("../../conformance/segment-store/v1/one-zero-bundle-catalog.hex");
const CATALOG_HEAD_HEX: &str =
    include_str!("../../conformance/segment-store/v1/one-zero-bundle-head.hex");

/// Decodes the frozen root transport.
///
/// # Errors
///
/// Returns the exact fixture transport refusal.
pub fn root_bytes() -> Result<Vec<u8>, Box<dyn Error>> {
    fixture(ROOT_HEX)
}

/// Decodes the frozen manifest transport.
///
/// # Errors
///
/// Returns the exact fixture transport refusal.
pub fn manifest_bytes() -> Result<Vec<u8>, Box<dyn Error>> {
    fixture(MANIFEST_HEX)
}

/// Builds the exact semantic successor of one admitted root.
///
/// # Errors
///
/// Returns the exact generation, semantic-root, or encoding refusal.
pub fn successor_root(
    current: &AdmittedRetentionRoot<'_>,
) -> Result<CanonicalRetentionRoot, Box<dyn Error>> {
    let root = RetentionRoot::new(
        current.root().namespace().clone(),
        current.root().generation().successor()?,
        RetentionPolicy::new(current.root().profile(), current.root().limits()),
        Some(current.digest()),
        current.root().anchors().to_vec(),
    )?;
    CanonicalRetentionRoot::from_root(&root).map_err(Into::into)
}

/// Builds a generation-one root for another namespace.
///
/// # Errors
///
/// Returns the exact namespace, semantic-root, or encoding refusal.
pub fn initial_root(
    namespace: &[u8],
    template: &AdmittedRetentionRoot<'_>,
) -> Result<CanonicalRetentionRoot, Box<dyn Error>> {
    let root = RetentionRoot::new(
        RetentionNamespace::try_from(namespace)?,
        keep::RootGeneration::INITIAL,
        RetentionPolicy::new(template.root().profile(), template.root().limits()),
        None,
        template.root().anchors().to_vec(),
    )?;
    CanonicalRetentionRoot::from_root(&root).map_err(Into::into)
}

/// Runs one operation against the frozen one-zero catalog snapshot.
///
/// # Errors
///
/// Returns the exact fixture, segment, catalog, head, or snapshot refusal.
pub fn with_snapshot<T>(
    operation: impl FnOnce(&CatalogSnapshot<'_, '_, '_>) -> T,
) -> Result<T, Box<dyn Error>> {
    let segment_bytes = fixture(SEGMENT_HEX)?;
    let catalog_bytes = fixture(CATALOG_HEX)?;
    let head_bytes = fixture(CATALOG_HEAD_HEX)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_policy())?;
    let segments = [segment];
    let catalog = admitted_catalog(&catalog_bytes, &segments)?;
    let head = ChecksummedPublicationHead::decode(&head_bytes)?;
    let snapshot = head.admit(catalog)?;
    Ok(operation(&snapshot))
}

/// Decodes one LF-terminated lowercase hexadecimal fixture.
///
/// # Errors
///
/// Returns a framing or hexadecimal transport refusal.
pub fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(hex.strip_suffix('\n').ok_or("fixture must end in one LF")?).map_err(Into::into)
}

fn admitted_catalog<'catalog, 'records>(
    catalog_bytes: &'catalog [u8],
    segments: &'records [AdmittedSegment<'records>],
) -> Result<AdmittedCatalog<'catalog, 'records>, Box<dyn Error>> {
    ChecksummedCatalog::decode(catalog_bytes)?
        .admit(segments)
        .map_err(Into::into)
}

const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}
