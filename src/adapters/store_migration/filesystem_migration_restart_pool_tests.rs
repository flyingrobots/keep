//! Restart admission verifies immutable pools and current HEAD before recovery.

use super::filesystem_migration_restart_test_fixture::{prefix, refusal};
use super::{
    FilesystemMigrationAuthorityError as Authority, FilesystemMigrationInventoryError as Inventory,
    MigrationInventoryPool as Pool, StoreMigrationPhase as Phase,
};
use crate::adapters::{
    CatalogDecodeError, CatalogRestartError, PublicationHeadDecodeError, SegmentHeaderError,
    SegmentReadError,
};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

// Size: medium. Oracle: malformed magic retains its exact decoder coordinates;
// restart cannot mutate a store whose immutable segment or catalog is corrupt.
// Delete only if immutable-pool verification is removed or stronger restart laws subsume it.
#[test]
fn damaged_immutable_pool_bytes_refuse_restart_before_effects() -> Result<(), Box<dyn Error>> {
    for (directory, pool) in [("segments", Pool::Segments), ("catalogs", Pool::Catalogs)] {
        let store = prefix(
            &format!("restart-damaged-{directory}"),
            Phase::RemoveIntentStage,
        )?;
        let path = only_entry(&store.path().join(directory))?;
        let (expected, observed) = damage_magic(&path)?;
        let error = refusal(store.path())?;
        let Some(Authority::Inventory {
            source:
                Inventory::Artifact {
                    pool: found,
                    source,
                    ..
                },
        }) = error.downcast_ref::<Authority>()
        else {
            return Err(format!("{directory}: wrong restart failure: {error:?}").into());
        };
        assert_eq!(
            *found, pool,
            "refusal must identify the damaged immutable pool"
        );
        assert!(
            match (pool, source.as_ref()) {
                (Pool::Segments, CatalogRestartError::Segment { source, .. }) =>
                    matches!(source.as_ref(),
                SegmentReadError::Header { source: SegmentHeaderError::InvalidMagic { expected: found_expected, observed: found_observed } }
                if *found_expected == expected && *found_observed == observed),
                (
                    Pool::Catalogs,
                    CatalogRestartError::Catalog {
                        source: CatalogDecodeError::InvalidMagic { observed: found },
                    },
                ) => *found == observed,
                _ => false,
            },
            "{directory}: exact decoder failure lost: {error:?}"
        );
        store.remove()?;
    }
    Ok(())
}

// Size: medium. Oracle: the freshly reopened current HEAD must decode before recovery.
// Delete only if migration no longer binds the version-one head.
#[test]
fn a_damaged_current_head_preserves_all_restart_evidence() -> Result<(), Box<dyn Error>> {
    let store = prefix("restart-damaged-current-head", Phase::RemoveIntentStage)?;
    let (_expected, observed) = damage_magic(&store.path().join("HEAD"))?;
    let error = refusal(store.path())?;
    assert!(
        matches!(error.downcast_ref::<Authority>(), Some(Authority::Head {
        source: PublicationHeadDecodeError::InvalidMagic { observed: found }
    }) if *found == observed),
        "HEAD refusal lost its decoder coordinate: {error:?}"
    );
    store.remove()?;
    Ok(())
}

fn only_entry(directory: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let mut entries = fs::read_dir(directory)?;
    let path = entries.next().ok_or("fixture pool is empty")??.path();
    if entries.next().is_some() {
        return Err("fixture pool is not a single artifact".into());
    }
    Ok(path)
}

type MagicCoordinates = ([u8; 16], [u8; 16]);

fn damage_magic(path: &Path) -> Result<MagicCoordinates, Box<dyn Error>> {
    let mut bytes = fs::read(path)?;
    let expected = bytes.get(..16).ok_or("no magic")?.try_into()?;
    *bytes.first_mut().ok_or("empty record")? ^= 1;
    let observed = bytes.get(..16).ok_or("no magic")?.try_into()?;
    fs::write(path, bytes)?;
    Ok((expected, observed))
}

// Size: medium. Oracle: valid alternate content cannot occupy another segment's name.
// Delete only if physical segment names no longer bind exact content.
#[test]
fn a_valid_segment_substitution_reports_both_digest_coordinates() -> Result<(), Box<dyn Error>> {
    use crate::adapters::{AdmittedSegment, SegmentReadPolicy};
    let store = prefix(
        "restart-valid-segment-substitution",
        Phase::RemoveIntentStage,
    )?;
    let path = only_entry(&store.path().join("segments"))?;
    let original = fs::read(&path)?;
    let replacement = super::filesystem_inventory_catalogs_test_fixture::empty_segment_bytes()?;
    let expected = AdmittedSegment::decode(&original, SegmentReadPolicy::MAXIMUM)?.digest();
    let observed = AdmittedSegment::decode(&replacement, SegmentReadPolicy::MAXIMUM)?.digest();
    fs::write(&path, replacement)?;
    let error = refusal(store.path())?;
    assert!(
        matches!(error.downcast_ref::<Authority>(), Some(Authority::Inventory {
        source: Inventory::Artifact { pool: Pool::Segments, source, .. }
    }) if matches!(source.as_ref(), CatalogRestartError::SegmentCoordinate { expected: found_expected, observed: found_observed }
        if *found_expected == expected && *found_observed == observed)),
        "{error:?}"
    );
    store.remove()?;
    Ok(())
}

// Size: medium. Oracle: persisted intent binds the entire immutable inventory, including orphans.
// Delete only if inventory binding is explicitly removed from the migration protocol.
#[test]
fn adding_a_valid_orphan_cannot_reuse_the_previous_migration_intent() -> Result<(), Box<dyn Error>>
{
    use super::{StoreMigrationRecoveryAmbiguity, StoreMigrationRecoveryError};
    use crate::adapters::{AdmittedSegment, SegmentReadPolicy, physical_pool_name};
    let store = prefix(
        "restart-valid-orphan-inventory-change",
        Phase::RemoveIntentStage,
    )?;
    let orphan = super::filesystem_inventory_catalogs_test_fixture::empty_segment_bytes()?;
    let digest = AdmittedSegment::decode(&orphan, SegmentReadPolicy::MAXIMUM)?.digest();
    fs::write(
        store
            .path()
            .join("segments")
            .join(physical_pool_name::segment(digest)),
        orphan,
    )?;
    let error = refusal(store.path())?;
    assert!(
        matches!(
            error.downcast_ref::<StoreMigrationRecoveryError>(),
            Some(StoreMigrationRecoveryError::Ambiguity {
                source: StoreMigrationRecoveryAmbiguity::IntentDiffers
            })
        ),
        "changed immutable inventory must not inherit old intent: {error:?}"
    );
    store.remove()?;
    Ok(())
}

// Size: medium. Oracle: canonical pool filenames have 68/85 bytes in segment-store/v1.
// Delete only if immutable pool naming changes by protocol decision.
#[test]
fn unknown_immutable_pool_names_refuse_with_the_exact_name() -> Result<(), Box<dyn Error>> {
    use crate::adapters::RecoveryPoolNameError;
    for (directory, pool, width) in [
        ("segments", Pool::Segments, 68),
        ("catalogs", Pool::Catalogs, 85),
    ] {
        let store = prefix(
            &format!("restart-unknown-pool-{directory}"),
            Phase::RemoveIntentStage,
        )?;
        fs::write(
            store.path().join(directory).join("unknown"),
            b"retained evidence",
        )?;
        let error = refusal(store.path())?;
        assert!(
            matches!(error.downcast_ref::<Authority>(), Some(Authority::Inventory {
            source: Inventory::Name { pool: found_pool, name, source: RecoveryPoolNameError::WrongLength { expected, observed: 7 } }
        }) if *found_pool == pool && name.as_bytes() == b"unknown" && *expected == width),
            "{directory}: {error:?}"
        );
        store.remove()?;
    }
    Ok(())
}
