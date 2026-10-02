//! Every publication crash prefix recovers to exactly one documented state.

use std::error::Error;
use std::fs;
use std::path::Path;

use super::filesystem_retention_test_fixture::{
    MANIFEST_HEX, PUBLICATION_PHASE_COUNT, ROOT_HEX, drive_publication, fixture,
    initial_preparation, open_authority, refusal, successor_preparation, successor_root,
};
use super::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, RetentionCurrentStateRefusal,
    RetentionPublicationError, RetentionPublicationOutcome, RetentionRecoveryOutcome as Outcome,
    RetentionRecoveryStep as Step,
};
use crate::execute_retention_publication;

const PROTECTED_ROOT: Outcome = Outcome::Protected {
    root_stage: true,
    manifest_stage: false,
};
const PROTECTED_BOTH: Outcome = Outcome::Protected {
    root_stage: true,
    manifest_stage: true,
};

/// The documented recovery of a crash after `count` phases, and the forward
/// retry's result afterwards.
fn expected(count: usize) -> (Vec<Step>, Outcome, Retry) {
    match count {
        0 | 1 => (vec![], Outcome::Clean, Retry::Published),
        2..=5 => (vec![Step::LinkRoot], PROTECTED_ROOT, Retry::Refused),
        6 | 7 => (vec![], PROTECTED_ROOT, Retry::Refused),
        8 | 9 => (vec![Step::LinkManifest], PROTECTED_BOTH, Retry::Refused),
        10 | 11 => (vec![], PROTECTED_BOTH, Retry::Refused),
        12 | 13 => (
            vec![
                Step::FinalizeHead,
                Step::RemoveRootStage,
                Step::RemoveManifestStage,
            ],
            Outcome::Committed,
            Retry::AlreadyCommitted,
        ),
        14 | 15 => (
            vec![Step::RemoveRootStage, Step::RemoveManifestStage],
            Outcome::Committed,
            Retry::AlreadyCommitted,
        ),
        16 => (
            vec![Step::RemoveManifestStage],
            Outcome::Committed,
            Retry::AlreadyCommitted,
        ),
        _ => (vec![], Outcome::Clean, Retry::AlreadyCommitted),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Retry {
    Published,
    AlreadyCommitted,
    Refused,
}

fn stages_present(root: &Path) -> [bool; 3] {
    let retention = root.join("retention");
    ["root.next", "manifest.next", "head.next"].map(|stage| retention.join(stage).exists())
}

fn stages_for(outcome: Outcome) -> [bool; 3] {
    match outcome {
        Outcome::Clean | Outcome::Committed => [false, false, false],
        Outcome::Protected {
            root_stage,
            manifest_stage,
        } => [root_stage, manifest_stage, false],
    }
}

#[test]
fn every_initial_publication_prefix_recovers_to_its_documented_state() -> Result<(), Box<dyn Error>>
{
    for count in 0..=PUBLICATION_PHASE_COUNT {
        let name = format!("filesystem-retention-recovery-prefix-{count}");
        let (sandbox, mut authority) = open_authority(&name)?;
        let root_bytes = fixture(ROOT_HEX)?;
        let preparation = initial_preparation(&root_bytes)?;
        drive_publication(&mut authority, &preparation, count)?;
        let (steps, outcome, retry) = expected(count);

        let receipt = authority
            .recover()
            .map_err(|error| format!("prefix {count}: {error}"))?;

        assert_eq!(receipt.executed(), steps, "prefix {count}: steps");
        assert_eq!(receipt.outcome(), outcome, "prefix {count}: outcome");
        assert_eq!(
            stages_present(sandbox.path()),
            stages_for(outcome),
            "prefix {count}: stages"
        );
        let second = authority
            .recover()
            .map_err(|error| format!("prefix {count} again: {error}"))?;
        assert!(
            second.executed().is_empty(),
            "prefix {count}: recovery is idempotent"
        );
        match (
            retry,
            execute_retention_publication(&mut authority, &preparation),
        ) {
            (Retry::Published, Ok(receipt)) => {
                assert_eq!(receipt.outcome(), RetentionPublicationOutcome::Published);
            }
            (Retry::AlreadyCommitted, Ok(receipt)) => {
                assert_eq!(
                    receipt.outcome(),
                    RetentionPublicationOutcome::AlreadyCommitted
                );
            }
            (Retry::Refused, Err(RetentionPublicationError::CurrentVerification { source })) => {
                assert!(
                    matches!(
                        refusal(&source),
                        Some(RetentionCurrentStateRefusal::RetainedStage)
                    ),
                    "prefix {count}: protected orphans refuse forward publication"
                );
            }
            (retry, result) => {
                return Err(format!("prefix {count}: expected {retry:?}, got {result:?}").into());
            }
        }
    }
    Ok(())
}

#[test]
fn a_crash_during_each_stage_write_requires_disposition_without_effects()
-> Result<(), Box<dyn Error>> {
    for (phase, stage, fixed) in [
        (2, "root.next", super::RetentionFixedStage::Root),
        (8, "manifest.next", super::RetentionFixedStage::Manifest),
        (12, "head.next", super::RetentionFixedStage::Head),
    ] {
        let name = format!("filesystem-retention-recovery-during-{phase}");
        let (sandbox, mut authority) = open_authority(&name)?;
        let root_bytes = fixture(ROOT_HEX)?;
        let preparation = initial_preparation(&root_bytes)?;
        drive_publication(&mut authority, &preparation, phase - 1)?;
        let publication = preparation.publication().ok_or("no publication")?;
        let complete: &[u8] = match phase {
            2 => preparation.candidate().encoded(),
            8 => publication.manifest().encoded(),
            _ => publication.head().encoded(),
        };
        // 100 bytes is inside every record's framing: the head is 144 bytes, the
        // manifest header 160, and the root header 192.
        let partial = complete.get(..100).ok_or("record shorter than 100 bytes")?;
        fs::write(sandbox.path().join("retention").join(stage), partial)?;
        let before = super::filesystem_retention_test_fixture::retention_witness(sandbox.path())?;
        let result = authority.recover();
        assert!(
            matches!(result, Err(super::FilesystemRetentionRecoveryError::Plan { source: super::RetentionRecoveryRefusal::IncompleteStageRequiresDisposition { stage: actual, observed: 100, .. } }) if actual == fixed),
            "during {phase}: typed disposition required: {result:?}"
        );
        assert_eq!(
            super::filesystem_retention_test_fixture::retention_witness(sandbox.path())?,
            before,
            "during {phase}: no recovery mutation"
        );
    }
    Ok(())
}

// Size: medium. Oracle: every ordered successor prefix preserves the old root until head finalization.
// Delete when stronger complete-prefix recovery evidence subsumes these runtime outcomes.
#[test]
fn successor_prefixes_recover_against_the_published_generation() -> Result<(), Box<dyn Error>> {
    for count in 0..=PUBLICATION_PHASE_COUNT {
        let name = format!("filesystem-retention-recovery-successor-{count}");
        let (sandbox, mut authority) = open_authority(&name)?;
        let root_bytes = fixture(ROOT_HEX)?;
        let _published =
            execute_retention_publication(&mut authority, &initial_preparation(&root_bytes)?)?;
        let current_root = AdmittedRetentionRoot::decode(&root_bytes)?;
        let manifest_bytes = fixture(MANIFEST_HEX)?;
        let current_manifest = AdmittedRetentionManifest::decode(&manifest_bytes)?;
        let candidate = successor_root(&current_root)?;
        let preparation =
            successor_preparation(&current_root, &current_manifest, candidate.encoded())?;
        drive_publication(&mut authority, &preparation, count)?;
        let (steps, outcome, _retry) = expected(count);

        let receipt = authority
            .recover()
            .map_err(|error| format!("successor {count}: {error}"))?;

        assert_eq!(receipt.executed(), steps, "successor {count}: steps");
        assert_eq!(receipt.outcome(), outcome, "successor {count}: outcome");
        let selected = if count < 12 {
            root_bytes.as_slice()
        } else {
            candidate.encoded()
        };
        require_successor_view(sandbox.path(), selected, count)?;
    }
    Ok(())
}

// Size: medium (owned filesystem); oracle: Core Law and recovery refusal
// contract. Delete only if the same public recovery refusal and exact-byte
// preservation are covered by a stronger test.
#[test]
fn noncanonical_short_stages_refuse_recovery_without_changing_retained_bytes()
-> Result<(), Box<dyn Error>> {
    use super::filesystem_retention_test_fixture::retention_witness;
    use super::{FilesystemRetentionRecoveryError, RetentionFixedStage, RetentionRecoveryRefusal};

    for (name, expected_stage) in [
        ("root.next", RetentionFixedStage::Root),
        ("manifest.next", RetentionFixedStage::Manifest),
        ("head.next", RetentionFixedStage::Head),
    ] {
        let (sandbox, mut authority) = open_authority(&format!("short-corrupt-{name}"))?;
        let root_bytes = fixture(ROOT_HEX)?;
        let _published =
            execute_retention_publication(&mut authority, &initial_preparation(&root_bytes)?)?;
        fs::write(sandbox.path().join("retention").join(name), b"invalid")?;
        let before = retention_witness(sandbox.path())?;

        let result = authority.recover();

        assert!(
            matches!(
                result,
                Err(FilesystemRetentionRecoveryError::Plan {
                    source: RetentionRecoveryRefusal::StageCorrupt { stage, .. }
                }) if stage == expected_stage
            ),
            "noncanonical {name} must refuse recovery, observed {result:?}"
        );
        assert_eq!(
            retention_witness(sandbox.path())?,
            before,
            "refusal must preserve every retained byte for {name}"
        );
    }
    Ok(())
}

/// Verifies persistent selected bytes independently of the recovery receipt.
fn require_successor_view(
    root: &Path,
    expected: &[u8],
    prefix: usize,
) -> Result<(), Box<dyn Error>> {
    use crate::{
        CatalogRestartByteLimit, CatalogRestartPolicy, FilesystemRetentionSnapshot,
        ReaderAttemptLimit, SegmentReadPolicy,
    };
    let policy = CatalogRestartPolicy::new(
        SegmentReadPolicy::MAXIMUM,
        CatalogRestartByteLimit::new(1_048_576)?,
    );
    let view = FilesystemRetentionSnapshot::load(root, policy, ReaderAttemptLimit::DEFAULT)?;
    let expected_root = AdmittedRetentionRoot::decode(expected)?;
    let head = view
        .retention_head()
        .ok_or("successor recovery lost the published head")?;
    let expected_generation = if prefix < 12 { 1 } else { 2 };
    assert_eq!(
        head.generation().get(),
        expected_generation,
        "successor prefix {prefix}: published generation"
    );
    let selected = view
        .retained_root(expected_root.root().namespace().digest())?
        .ok_or("successor recovery lost the selected root")?;
    assert_eq!(
        &*selected, expected,
        "successor prefix {prefix}: exact selected root bytes"
    );
    Ok(())
}
