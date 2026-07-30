//! Deterministic retention-closure verification laws.

mod support;

use std::error::Error;

use blake3::Hasher;
use keep::{
    AdmittedCatalog, AdmittedSegment, BlobId, CatalogSnapshot, ChecksummedCatalog,
    ChecksummedPublicationHead, LayoutEntryLimit, LayoutId, RegisteredRetentionProfile,
    RetentionAnchor, RetentionClosureLimits, RetentionNamespace, RetentionPolicy, RetentionRoot,
    RootGeneration, SegmentReadPolicy, SegmentRecordLimit, verify_retention_closure,
};
use support::decode_hex;

const BUNDLE_SEGMENT_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-bundle-segment.hex");
const BUNDLE_CATALOG_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-bundle-catalog.hex");
const BUNDLE_HEAD_HEX: &str =
    include_str!("../conformance/segment-store/v1/one-zero-bundle-head.hex");
const ONE_ZERO_BLOB: &str = concat!(
    "keep:blob:v1:blake3-256:1:",
    "1cfb8fa9e917aba15a1f592095f377ff180755fe1212b0d7d2ec750bd128b606"
);
const ONE_ZERO_LAYOUT: &str = concat!(
    "keep:layout:v1:flat-chunks-v1:blake3-256:220:",
    "887da23f1a7483359a78fc9a7fde80030ec2c4690603803f0ab7d0edb56575b8"
);
const CHUNK_DIGEST_HEX: &str = "9b9c9a42912a0efdcd41e83ea024d72f10f2627d239e4eb240dd53f39ce0ff62";
const CHUNK_RECORD_CHECKSUM_HEX: &str =
    "becb46b35120723210798a47e26144b8214d5ea65d28806e0ba941d2aa66bbfa";
const LAYOUT_RECORD_CHECKSUM_HEX: &str =
    "c498a9c3cc24142926857d778fee7fd622b8b03312318a2360a68be3461168d6";
const CLOSURE_DOMAIN: &[u8] = b"keep.retention-closure/v2\0";

#[test]
fn one_anchor_closure_binds_exact_evidence_and_authenticated_bytes() -> Result<(), Box<dyn Error>> {
    let segment_bytes = fixture(BUNDLE_SEGMENT_HEX)?;
    let catalog_bytes = fixture(BUNDLE_CATALOG_HEX)?;
    let head_bytes = fixture(BUNDLE_HEAD_HEX)?;
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_policy())?;
    let segments = [segment];
    let catalog = admitted_catalog(&catalog_bytes, &segments)?;
    let head = ChecksummedPublicationHead::decode(&head_bytes)?;
    let snapshot = head.admit(catalog)?;
    let root = one_anchor_root()?;

    let evidence = verify_retention_closure(&root, &snapshot)?;

    assert_eq!(evidence.profile(), root.profile());
    assert_eq!(evidence.catalog_generation(), snapshot.generation());
    assert_eq!(evidence.catalog_digest(), snapshot.catalog_digest());
    assert_eq!(evidence.usage().node_count(), 2);
    assert_eq!(evidence.usage().maximum_depth(), 2);
    assert_eq!(evidence.usage().encoded_bytes(), 220);
    assert_eq!(evidence.usage().physical_bytes(), 509);
    assert_eq!(evidence.digest().as_bytes(), &expected_digest(&snapshot)?);
    Ok(())
}

fn admitted_catalog<'catalog, 'records>(
    catalog_bytes: &'catalog [u8],
    segments: &'records [AdmittedSegment<'records>],
) -> Result<AdmittedCatalog<'catalog, 'records>, Box<dyn Error>> {
    ChecksummedCatalog::decode(catalog_bytes)?
        .admit(segments)
        .map_err(Into::into)
}

fn one_anchor_root() -> Result<RetentionRoot, Box<dyn Error>> {
    let blob: BlobId = ONE_ZERO_BLOB.parse()?;
    let layout: LayoutId = ONE_ZERO_LAYOUT.parse()?;
    Ok(RetentionRoot::new(
        RetentionNamespace::try_from(b"contract".as_slice())?,
        RootGeneration::new(1)?,
        RetentionPolicy::new(
            RegisteredRetentionProfile::SINGLE_CANONICAL_WITNESS_V1,
            RetentionClosureLimits::new(2, 2, 220, 509)?,
        ),
        None,
        vec![RetentionAnchor::new(blob, layout)],
    )?)
}

fn expected_digest(snapshot: &CatalogSnapshot<'_, '_, '_>) -> Result<[u8; 32], Box<dyn Error>> {
    let profile = RegisteredRetentionProfile::SINGLE_CANONICAL_WITNESS_V1;
    let mut hasher = Hasher::new();
    hasher.update(CLOSURE_DOMAIN);
    hasher.update(&profile.identity().to_be_bytes());
    hasher.update(&profile.version().to_be_bytes());
    hasher.update(profile.digest());
    hasher.update(&snapshot.generation().get().to_be_bytes());
    hasher.update(snapshot.catalog_digest().as_bytes());
    hasher.update(&2_u64.to_be_bytes());
    hasher.update(&2_u16.to_be_bytes());
    hasher.update(&[0_u8; 6]);
    hasher.update(&220_u64.to_be_bytes());
    hasher.update(&509_u64.to_be_bytes());
    hasher.update(&chunk_member()?);
    hasher.update(&layout_member()?);
    Ok(*hasher.finalize().as_bytes())
}

fn chunk_member() -> Result<[u8; 96], Box<dyn Error>> {
    let mut entry = [0_u8; 96];
    *entry.first_mut().ok_or("closure member has no kind byte")? = 1;
    entry
        .get_mut(4..8)
        .ok_or("closure member lacks chunk length")?
        .copy_from_slice(&1_u32.to_be_bytes());
    entry
        .get_mut(8..40)
        .ok_or("closure member lacks chunk digest")?
        .copy_from_slice(&digest(CHUNK_DIGEST_HEX)?);
    entry
        .get_mut(64..96)
        .ok_or("closure member lacks checksum")?
        .copy_from_slice(&digest(CHUNK_RECORD_CHECKSUM_HEX)?);
    Ok(entry)
}

fn layout_member() -> Result<[u8; 96], Box<dyn Error>> {
    let layout: LayoutId = ONE_ZERO_LAYOUT.parse()?;
    let mut entry = [0_u8; 96];
    *entry.first_mut().ok_or("closure member has no kind byte")? = 2;
    entry
        .get_mut(4..64)
        .ok_or("closure member lacks layout identity")?
        .copy_from_slice(&layout.encode_binary());
    entry
        .get_mut(64..96)
        .ok_or("closure member lacks checksum")?
        .copy_from_slice(&digest(LAYOUT_RECORD_CHECKSUM_HEX)?);
    Ok(entry)
}

fn digest(hex: &str) -> Result<[u8; 32], Box<dyn Error>> {
    decode_hex(hex)?
        .try_into()
        .map_err(|_source| "digest fixture is not 32 bytes".into())
}

const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}

fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(hex.strip_suffix('\n').ok_or("fixture must end in one LF")?).map_err(Into::into)
}
