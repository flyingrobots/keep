//! Exact recovery-name ambiguity laws.

use std::error::Error;

use keep::{
    RecoveryEntryRole, RecoveryNameClassificationError, RecoveryNamespace, RecoveryPoolNameError,
    RecoveryRequiredEntry, classify_recovery_names,
};

use super::{initialized_root, inventory, name, names};

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
        names(&["staging", "segments", "catalogs"])?,
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
