//! Complete invalid generation fields cannot authorize stage evidence deletion.
//! Size: medium; oracle: positive generation and mutation-free refusal contracts.
//! Delete only if a stronger public recovery test subsumes this preservation law.

use std::error::Error;
use std::fs;

use super::filesystem_retention_test_fixture::{
    HEAD_HEX, MANIFEST_HEX, ROOT_HEX, fixture, initial_preparation, open_authority,
    retention_witness,
};
use super::{FilesystemRetentionRecoveryError, RetentionFixedStage, RetentionRecoveryRefusal};
use crate::execute_retention_publication;

#[test]
fn invalid_complete_generations_refuse_recovery_without_changing_retained_bytes()
-> Result<(), Box<dyn Error>> {
    for (name, stage, hex, generation) in [
        ("root.next", RetentionFixedStage::Root, ROOT_HEX, 32..40),
        (
            "manifest.next",
            RetentionFixedStage::Manifest,
            MANIFEST_HEX,
            32..40,
        ),
        ("head.next", RetentionFixedStage::Head, HEAD_HEX, 24..32),
    ] {
        let (sandbox, mut authority) = open_authority(&format!("zero-generation-{name}"))?;
        let root_bytes = fixture(ROOT_HEX)?;
        let _published =
            execute_retention_publication(&mut authority, &initial_preparation(&root_bytes)?)?;
        let mut bytes = fixture(hex)?;
        let end = generation.end;
        bytes
            .get_mut(generation)
            .ok_or("missing generation")?
            .fill(0);
        let prefix = bytes.get(..end).ok_or("missing stage prefix")?;
        fs::write(sandbox.path().join("retention").join(name), prefix)?;
        let before = retention_witness(sandbox.path())?;

        let result = authority.recover();

        assert!(
            matches!(result, Err(FilesystemRetentionRecoveryError::Plan {
            source: RetentionRecoveryRefusal::StageCorrupt { stage: observed, .. }
        }) if observed == stage),
            "zero-generation {name} must refuse, observed {result:?}"
        );
        assert_eq!(
            retention_witness(sandbox.path())?,
            before,
            "generation refusal must preserve every retained byte for {name}"
        );
    }
    Ok(())
}
