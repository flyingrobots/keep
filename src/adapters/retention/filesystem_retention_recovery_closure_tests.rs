//! These laws own transitive catalog admission before recovery publication.

use std::error::Error;
use std::fs;
use std::io;

use super::FilesystemRetentionRecoveryError;
use super::filesystem_retention_test_fixture::{
    CATALOG_NAME, ROOT_HEX, SEGMENT_NAME, drive_publication, fixture, initial_preparation,
    open_authority, retention_witness,
};
use crate::adapters::{
    CatalogDecodeError, CatalogRestartError, CatalogRestartPhase, SegmentReadError,
    SegmentSealError,
};

#[derive(Clone, Copy, Debug)]
enum Damage {
    MissingSegment,
    CorruptSegment,
    MissingCatalog,
    CorruptCatalog,
}

// Size: medium. Oracle: complete recovery stages require authenticated transitive members.
// Delete only with the retention protocol or a stronger runtime replacement.
#[test]
fn recovery_refuses_missing_segments_before_root_link() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::MissingSegment, 2)
}
#[test]
fn recovery_refuses_missing_segments_before_manifest_link() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::MissingSegment, 8)
}
#[test]
fn recovery_refuses_missing_segments_before_head_commit() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::MissingSegment, 13)
}
#[test]
fn recovery_refuses_corrupt_segments_before_root_link() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::CorruptSegment, 2)
}
#[test]
fn recovery_refuses_corrupt_segments_before_manifest_link() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::CorruptSegment, 8)
}
#[test]
fn recovery_refuses_corrupt_segments_before_head_commit() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::CorruptSegment, 13)
}
#[test]
fn recovery_refuses_missing_catalogs_before_root_link() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::MissingCatalog, 2)
}
#[test]
fn recovery_refuses_missing_catalogs_before_manifest_link() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::MissingCatalog, 8)
}
#[test]
fn recovery_refuses_missing_catalogs_before_head_commit() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::MissingCatalog, 13)
}
#[test]
fn recovery_refuses_corrupt_catalogs_before_root_link() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::CorruptCatalog, 2)
}
#[test]
fn recovery_refuses_corrupt_catalogs_before_manifest_link() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::CorruptCatalog, 8)
}
#[test]
fn recovery_refuses_corrupt_catalogs_before_head_commit() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::CorruptCatalog, 13)
}

fn require_refusal(damage: Damage, prefix: usize) -> Result<(), Box<dyn Error>> {
    let label = format!("recovery-closure-{damage:?}-{prefix}");
    let (sandbox, mut authority) = open_authority(&label)?;
    let bytes = fixture(ROOT_HEX)?;
    drive_publication(&mut authority, &initial_preparation(&bytes)?, prefix)?;
    let (directory, name) = match damage {
        Damage::MissingSegment | Damage::CorruptSegment => ("segments", SEGMENT_NAME),
        Damage::MissingCatalog | Damage::CorruptCatalog => ("catalogs", CATALOG_NAME),
    };
    let path = sandbox.path().join(directory).join(name);
    match damage {
        Damage::MissingSegment | Damage::MissingCatalog => fs::remove_file(path)?,
        Damage::CorruptSegment | Damage::CorruptCatalog => {
            let mut bytes = fs::read(&path)?;
            *bytes.last_mut().ok_or("empty transitive artifact")? ^= 1;
            fs::write(path, bytes)?;
        }
    }
    let before = retention_witness(sandbox.path())?;
    let result = authority.recover();
    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "closure refusal must preserve retention evidence for {label}: {result:?}"
    );
    let error = match &result {
        Err(FilesystemRetentionRecoveryError::Observe { source }) => source,
        other => {
            return Err(format!(
                "expected typed closure observation refusal for {label}: {other:?}"
            )
            .into());
        }
    };
    let restart = error
        .get_ref()
        .and_then(|source| source.downcast_ref::<CatalogRestartError>());
    assert!(
        matches_damage(damage, restart),
        "closure refusal must preserve exact {damage:?} source for {label}: {result:?}"
    );
    Ok(())
}

fn matches_damage(damage: Damage, error: Option<&CatalogRestartError>) -> bool {
    match (damage, error) {
        (Damage::MissingSegment, Some(CatalogRestartError::Io { phase, source })) => {
            *phase == CatalogRestartPhase::OpenSegment && source.kind() == io::ErrorKind::NotFound
        }
        (Damage::MissingCatalog, Some(CatalogRestartError::Io { phase, source })) => {
            *phase == CatalogRestartPhase::OpenCatalog && source.kind() == io::ErrorKind::NotFound
        }
        (Damage::CorruptCatalog, Some(CatalogRestartError::Catalog { source })) => {
            matches!(source, CatalogDecodeError::ChecksumMismatch { .. })
        }
        (Damage::CorruptSegment, Some(CatalogRestartError::Segment { source, .. })) => matches!(
            source.as_ref(),
            SegmentReadError::Seal {
                source: SegmentSealError::SealChecksumMismatch { .. }
            }
        ),
        _ => false,
    }
}
