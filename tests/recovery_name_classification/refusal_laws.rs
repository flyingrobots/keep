//! Exact recovery-name ambiguity laws.

use std::error::Error;

use keep::{
    RecoveryEntryRole, RecoveryNameClassificationError, RecoveryNamespace, RecoveryPoolNameError,
    RecoveryRequiredEntry, classify_recovery_names,
};

use super::{INITIALIZED_ROOT, initialized_root, inventory, name, names};

#[test]
fn unknown_name_is_refused_with_its_exact_namespace_and_bytes() -> Result<(), Box<dyn Error>> {
    let Err(error) = classify_recovery_names(inventory([
        initialized_root()?,
        Vec::new(),
        names(&["not-a-segment"])?,
        Vec::new(),
    ])?) else {
        return Err("name classification admitted an unknown segment entry".into());
    };

    assert!(matches!(
        error,
        RecoveryNameClassificationError::PoolName {
            namespace: RecoveryNamespace::Segments,
            ref name,
            source: RecoveryPoolNameError::WrongLength {
                expected: 68,
                observed: 13,
            },
        } if name.as_bytes() == b"not-a-segment"
    ));
    Ok(())
}

#[test]
fn missing_initialized_root_entry_is_an_exact_refusal() -> Result<(), Box<dyn Error>> {
    let Err(error) = classify_recovery_names(inventory([
        names(
            INITIALIZED_ROOT
                .get(1..)
                .ok_or("initialized root is empty")?,
        )?,
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ])?) else {
        return Err("name classification admitted a missing writer lock".into());
    };

    assert!(matches!(
        error,
        RecoveryNameClassificationError::Missing {
            required: RecoveryRequiredEntry::WriterLock,
        }
    ));
    Ok(())
}

#[test]
fn uppercase_pool_digest_is_noncanonical() -> Result<(), Box<dyn Error>> {
    let Err(error) = classify_recovery_names(inventory([
        initialized_root()?,
        Vec::new(),
        vec![name(format!("{}.seg", "AA".repeat(32)).as_bytes())?],
        Vec::new(),
    ])?) else {
        return Err("name classification admitted uppercase pool identity".into());
    };

    assert!(matches!(
        error,
        RecoveryNameClassificationError::PoolName {
            source: RecoveryPoolNameError::UppercaseDigest,
            ..
        }
    ));
    Ok(())
}

#[test]
fn simultaneous_fixed_stages_are_unrecoverable_ambiguity() -> Result<(), Box<dyn Error>> {
    let Err(error) = classify_recovery_names(inventory([
        initialized_root()?,
        names(&["current.seg", "current.cat"])?,
        Vec::new(),
        Vec::new(),
    ])?) else {
        return Err("name classification admitted simultaneous fixed stages".into());
    };

    assert!(matches!(
        error,
        RecoveryNameClassificationError::ConflictingStages {
            first: RecoveryEntryRole::CatalogStage,
            second: RecoveryEntryRole::SegmentStage,
        }
    ));
    Ok(())
}

/// Every root entry of an initialized store is required, so a root that lacks
/// one of them, including a former version-1 root without `FORMAT` or
/// `reader.lock`, refuses as exactly that missing entry.
#[test]
fn every_initialized_root_entry_is_required() -> Result<(), Box<dyn Error>> {
    let required = [
        RecoveryRequiredEntry::WriterLock,
        RecoveryRequiredEntry::StagingDirectory,
        RecoveryRequiredEntry::SegmentPoolDirectory,
        RecoveryRequiredEntry::CatalogPoolDirectory,
        RecoveryRequiredEntry::ReaderFence,
        RecoveryRequiredEntry::FormatMarker,
        RecoveryRequiredEntry::RetentionDirectory,
        RecoveryRequiredEntry::GcDirectory,
        RecoveryRequiredEntry::RecoveryDirectory,
    ];
    for (missing, expected) in INITIALIZED_ROOT.iter().zip(required) {
        let present: Vec<&str> = INITIALIZED_ROOT
            .iter()
            .copied()
            .filter(|entry| entry != missing)
            .collect();
        let Err(error) = classify_recovery_names(inventory([
            names(&present)?,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ])?) else {
            return Err(format!("classification admitted a root without {missing}").into());
        };
        assert!(
            matches!(
                error,
                RecoveryNameClassificationError::Missing { required } if required == expected
            ),
            "a root without {missing} refused as {error:?}"
        );
    }
    Ok(())
}
