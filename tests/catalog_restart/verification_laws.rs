//! Verification classification at the existing filesystem catalog boundary.
use super::{StoreFixture, restart_policy};
use keep::{
    CatalogRestartByteLimit, CatalogRestartError, CatalogRestartPhase, CatalogRestartPolicy,
    FilesystemCatalogSnapshot, PublicationHeadDecodeError, SegmentReadPolicy, VerificationDepth,
    VerificationError, VerificationRefusal, VerificationSource, VerificationSubject,
};
use std::error::Error;
use std::fs;

// Size: medium. Oracle: exact frozen catalog/head and unchanged physical bytes.
// Delete if superseded by generated owned-view verification over the same corpus.
#[test]
fn owned_catalog_verification_binds_admitted_immutable_bytes() -> Result<(), Box<dyn Error>> {
    let store = StoreFixture::create("verification-catalog")?;
    let before = fs::read(&store.catalog_path)?;
    let view = FilesystemCatalogSnapshot::load_for_verification(store.path(), restart_policy()?)?;
    let report = view.verify(VerificationDepth::CatalogReachability)?;
    assert_eq!(
        report.catalog(),
        Some((view.generation(), view.catalog_digest()))
    );
    assert_eq!(
        report
            .subjects()
            .first()
            .ok_or("missing catalog claim")?
            .subject(),
        VerificationSubject::Catalog {
            generation: view.generation(),
            digest: view.catalog_digest()
        }
    );
    assert_eq!(fs::read(&store.catalog_path)?, before);
    store.remove()
}

// Size: medium. Oracle: deleting the named catalog establishes missing evidence.
// Delete if a stronger deterministic fault law subsumes this boundary.
#[test]
fn verification_does_not_relabel_a_missing_catalog_as_corrupt() -> Result<(), Box<dyn Error>> {
    let store = StoreFixture::create("verification-missing-catalog")?;
    fs::remove_file(&store.catalog_path)?;
    let error = FilesystemCatalogSnapshot::load_for_verification(store.path(), restart_policy()?)
        .err()
        .ok_or("missing catalog admitted")?;
    assert!(
        matches!(error, VerificationError::Refused { refusal: VerificationRefusal::Missing { subject: VerificationSubject::PublishedCatalog }, source: Some(source) } if matches!(source.as_ref(), VerificationSource::Catalog(CatalogRestartError::Io { phase: CatalogRestartPhase::OpenCatalog, source }) if source.kind() == std::io::ErrorKind::NotFound)),
        "missing selected catalog must retain its opening cause"
    );
    store.remove()
}

// Size: medium. Oracle: changed header version preserves both typed coordinates.
// Delete if report-level protocol mutation coverage subsumes this witness.
#[test]
fn catalog_verification_retains_the_original_head_protocol_refusal() -> Result<(), Box<dyn Error>> {
    let store = StoreFixture::create("verification-corrupt-head")?;
    let path = store.path().join("HEAD");
    let mut bytes = fs::read(&path)?;
    *bytes.get_mut(17).ok_or("version field absent")? = 2;
    fs::write(&path, &bytes)?;
    let error = FilesystemCatalogSnapshot::load_for_verification(store.path(), restart_policy()?)
        .err()
        .ok_or("unsupported head admitted")?;
    assert!(
        matches!(error, VerificationError::Refused { refusal: VerificationRefusal::Corrupt { .. }, source: Some(source) } if matches!(source.as_ref(), VerificationSource::Catalog(CatalogRestartError::Head { source: PublicationHeadDecodeError::UnsupportedVersion { expected: 1, observed: 2 } }))),
        "exact protocol refusal must survive classification"
    );
    assert_eq!(
        fs::read(&path)?,
        bytes,
        "verification must preserve corrupt evidence"
    );
    store.remove()
}

// Size: medium. Oracle: configured retained bytes constrain resources, not truth.
// Delete if a stronger memory-budget law subsumes this failure boundary.
#[test]
fn catalog_memory_limits_do_not_become_corruption_claims() -> Result<(), Box<dyn Error>> {
    let store = StoreFixture::create("verification-catalog-capacity")?;
    let policy =
        CatalogRestartPolicy::new(SegmentReadPolicy::MAXIMUM, CatalogRestartByteLimit::new(1)?);
    let error = FilesystemCatalogSnapshot::load_for_verification(store.path(), policy)
        .err()
        .ok_or("byte limit ignored")?;
    assert!(
        matches!(error, VerificationError::Operational { source } if matches!(source.as_ref(), VerificationSource::Catalog(CatalogRestartError::RetainedSegmentBytes { maximum: 1, observed: 337 }))),
        "exact bounded loading refusal must survive"
    );
    store.remove()
}
