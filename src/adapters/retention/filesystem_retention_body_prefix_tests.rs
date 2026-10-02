//! This module owns preservation of invalid complete entries in interrupted retention bodies.

use super::filesystem_retention_test_fixture::{
    MANIFEST_HEX, ROOT_HEX, drive_publication, fixture, initial_preparation, open_authority,
    retention_witness,
};
use super::{
    FilesystemRetentionRecoveryError, RetentionFixedStage as Stage, RetentionManifestDecodeError,
    RetentionRecoveryRefusal, RetentionRootDecodeError,
};
use crate::{BlobIdBinaryParseError, LayoutIdBinaryParseError, RootGenerationError};
use std::{error::Error, fs};

#[derive(Clone, Copy, Debug)]
enum Fault {
    Blob,
    Layout,
    Generation,
    Order,
}

// Size: medium. Oracle: complete identities and root generations obey their public admission rules.
// Delete only when stronger recovery coverage subsumes incomplete bodies containing invalid entries.
#[test]
fn invalid_complete_entries_in_short_bodies_preserve_evidence() -> Result<(), Box<dyn Error>> {
    for (stage, fault) in [
        (Stage::Root, Fault::Blob),
        (Stage::Root, Fault::Layout),
        (Stage::Manifest, Fault::Generation),
    ] {
        require_preservation(stage, fault)?;
    }
    Ok(())
}

// Size: medium. Oracle: root anchors and manifest namespaces are strictly ordered without duplicates.
// Delete only when stronger recovery coverage subsumes duplicate entries before body completion.
#[test]
fn duplicate_entries_in_short_bodies_preserve_evidence() -> Result<(), Box<dyn Error>> {
    for stage in [Stage::Root, Stage::Manifest] {
        require_preservation(stage, Fault::Order)?;
    }
    Ok(())
}

fn malformed_prefix(stage: Stage, fault: Fault) -> Result<Vec<u8>, Box<dyn Error>> {
    let (corpus, width) = match stage {
        Stage::Root => (ROOT_HEX, 119_usize),
        _ => (MANIFEST_HEX, 72),
    };
    let bytes = fixture(corpus)?;
    let body_end = bytes.len().checked_sub(64).ok_or("missing trailer")?;
    let body_start = body_end.checked_sub(width).ok_or("missing body")?;
    let mut prefix = bytes.get(..body_end).ok_or("missing prefix")?.to_vec();
    let count = if matches!(fault, Fault::Order) {
        3_u32
    } else {
        2
    };
    if matches!(fault, Fault::Order) {
        prefix.extend_from_slice(bytes.get(body_start..body_end).ok_or("missing entry")?);
    }
    let extra = usize::try_from(count.checked_sub(1).ok_or("count underflow")?)?
        .checked_mul(width)
        .ok_or("length overflow")?;
    let total = u64::try_from(bytes.len().checked_add(extra).ok_or("length overflow")?)?;
    prefix
        .get_mut(24..32)
        .ok_or("missing length")?
        .copy_from_slice(&total.to_be_bytes());
    prefix
        .get_mut(44..48)
        .ok_or("missing count")?
        .copy_from_slice(&count.to_be_bytes());
    match fault {
        Fault::Blob => *prefix.get_mut(body_start).ok_or("missing blob magic")? = 0,
        Fault::Layout => {
            *prefix
                .get_mut(body_start.checked_add(59).ok_or("offset overflow")?)
                .ok_or("missing layout magic")? = 0
        }
        Fault::Generation => {
            let start = body_start.checked_add(32).ok_or("offset overflow")?;
            let end = start.checked_add(8).ok_or("offset overflow")?;
            prefix
                .get_mut(start..end)
                .ok_or("missing generation")?
                .fill(0);
        }
        Fault::Order => {}
    }
    Ok(prefix)
}

fn require_preservation(stage: Stage, fault: Fault) -> Result<(), Box<dyn Error>> {
    let (name, phases) = match stage {
        Stage::Root => ("root.next", 1),
        _ => ("manifest.next", 7),
    };
    let (sandbox, mut authority) = open_authority(&format!("body-prefix-{name}-{fault:?}"))?;
    let root = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root)?;
    drive_publication(&mut authority, &preparation, phases)?;
    fs::write(
        sandbox.path().join("retention").join(name),
        malformed_prefix(stage, fault)?,
    )?;
    let before = retention_witness(sandbox.path())?;
    match authority.recover() {
        Err(FilesystemRetentionRecoveryError::Plan {
            source:
                RetentionRecoveryRefusal::StageCorrupt {
                    stage: actual,
                    source,
                },
        }) => {
            assert_eq!(actual, stage, "identify the invalid body stage");
            require_cause(source.as_ref(), stage, fault);
        }
        result => {
            return Err(
                format!("{name} {fault:?} must refuse before body completion: {result:?}").into(),
            );
        }
    }
    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "invalid body entry must preserve retained evidence"
    );
    Ok(())
}

fn require_cause(source: &(dyn Error + 'static), stage: Stage, fault: Fault) {
    let mut blob_magic = *b"KEEP:BLOB:ID\0\0\0\0";
    let mut layout_magic = *b"KEEP:LAYOUT:ID\0\0";
    if let Some(first) = blob_magic.first_mut() {
        *first = 0;
    }
    if let Some(first) = layout_magic.first_mut() {
        *first = 0;
    }
    let exact = match (stage, fault) {
        (Stage::Root, Fault::Blob) => {
            matches!(source.downcast_ref::<RetentionRootDecodeError>(), Some(RetentionRootDecodeError::BlobId { index: 0, source: BlobIdBinaryParseError::InvalidMagic { observed } }) if *observed == blob_magic)
        }
        (Stage::Root, Fault::Layout) => {
            matches!(source.downcast_ref::<RetentionRootDecodeError>(), Some(RetentionRootDecodeError::LayoutId { index: 0, source: LayoutIdBinaryParseError::InvalidMagic { observed } }) if *observed == layout_magic)
        }
        (Stage::Root, Fault::Order) => matches!(
            source.downcast_ref::<RetentionRootDecodeError>(),
            Some(RetentionRootDecodeError::NonCanonicalAnchorOrder { index: 1 })
        ),
        (Stage::Manifest, Fault::Generation) => matches!(
            source.downcast_ref::<RetentionManifestDecodeError>(),
            Some(RetentionManifestDecodeError::RootGeneration {
                index: 0,
                source: RootGenerationError::Zero
            })
        ),
        (Stage::Manifest, Fault::Order) => matches!(
            source.downcast_ref::<RetentionManifestDecodeError>(),
            Some(RetentionManifestDecodeError::NonCanonicalEntryOrder { index: 1 })
        ),
        _ => false,
    };
    assert!(
        exact,
        "report exact body-entry violation for {stage:?} {fault:?}: {source:?}"
    );
}
