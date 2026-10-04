//! This module owns namespace-size refusal before interrupted-root discard.

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, open_authority, retention_witness,
};
use super::{
    FilesystemRetentionRecoveryError, RetentionFixedStage, RetentionRecoveryRefusal,
    RetentionRootDecodeError,
};
use crate::RetentionNamespaceError;
use std::{error::Error, fs};

// Size: medium. Oracle: an opaque namespace has 1..=255 bytes regardless of missing payload.
// Delete only when stronger public recovery evidence covers these invalid size boundaries.
#[test]
fn impossible_namespace_sizes_in_short_roots_preserve_evidence() -> Result<(), Box<dyn Error>> {
    for size in [0_u16, 256, u16::MAX] {
        let expected = if size == 0 {
            RetentionNamespaceError::Empty
        } else {
            RetentionNamespaceError::TooLong {
                maximum: 255,
                observed: usize::from(size),
            }
        };
        for end in [42, 48] {
            let (sandbox, mut authority) =
                open_authority(&format!("short-namespace-{size}-{end}"))?;
            let bytes = fixture(ROOT_HEX)?;
            // The golden root contains a three-byte namespace; preserve consistent framing.
            let length = u64::try_from(bytes.len())?
                .checked_sub(3)
                .and_then(|length| length.checked_add(u64::from(size)))
                .ok_or("length overflow")?;
            let mut partial = bytes.get(..end).ok_or("short root fixture")?.to_vec();
            partial
                .get_mut(24..32)
                .ok_or("missing record length")?
                .copy_from_slice(&length.to_be_bytes());
            partial
                .get_mut(40..42)
                .ok_or("missing namespace length")?
                .copy_from_slice(&size.to_be_bytes());
            fs::write(sandbox.path().join("retention/root.next"), partial)?;
            let before = retention_witness(sandbox.path())?;

            let result = authority.recover();

            assert!(
                matches!(&result,
                Err(FilesystemRetentionRecoveryError::Plan {
                    source: RetentionRecoveryRefusal::StageCorrupt { stage: RetentionFixedStage::Root, source },
                }) if matches!(source.downcast_ref::<RetentionRootDecodeError>(),
                    Some(RetentionRootDecodeError::Namespace { source }) if *source == expected)),
                "namespace size {size}, prefix {end} must report {expected:?}: {result:?}"
            );
            assert_eq!(
                retention_witness(sandbox.path())?,
                before,
                "namespace size {size}, prefix {end} must preserve retained evidence"
            );
        }
    }
    Ok(())
}
