//! Explicit disposition laws over recovery-protected retention orphans.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use cap_std::fs::Dir;

use super::filesystem_retention_pool_name as pool_name;
use super::filesystem_retention_test_fixture::{
    MANIFEST_HEX, ROOT_HEX, catalog_policy, drive_publication, fixture, initial_preparation,
    initial_root, new_namespace_preparation, open_authority, refusal, reopen_authority,
    successor_preparation, successor_root,
};
use super::{
    AdmittedRetentionManifest, AdmittedRetentionRoot,
    FilesystemRetentionDispositionError as DispositionError, ReaderFence,
    RecoveryDispositionAmbiguity, RecoveryDispositionRefusal, RecoveryDispositionRequest,
    RecoveryDispositionTarget, RetentionCurrentStateRefusal, RetentionPublicationOutcome,
    RetentionPublicationStorage,
};
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::{
    AdmittedRecoveryDispositionReceipt, FilesystemRetentionSnapshot, GcRetentionState,
    ReaderAttemptLimit, RecoveryArtifactKind, RecoveryClassification, RecoveryDispositionDecision,
};
use crate::execute_retention_publication;

const ROOT_DIGEST_HEX: &str = "ca4c11f265c3bed07073bdc3b6aef003e964ac8cb36fcfcc92f20fa6f0b60085";

/// Publishes the first `count` phases of the initial root publication and
/// releases the writer, leaving a recovery-protected orphan behind.
fn interrupted_store(name: &str, count: usize) -> Result<TestDirectory, Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority(name)?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    drive_publication(&mut authority, &preparation, count)?;
    drop(authority);
    Ok(sandbox)
}

fn request(
    target: RecoveryDispositionTarget,
    decision: RecoveryDispositionDecision,
) -> RecoveryDispositionRequest {
    RecoveryDispositionRequest { target, decision }
}

const fn retire(target: RecoveryDispositionTarget) -> RecoveryDispositionRequest {
    RecoveryDispositionRequest {
        target,
        decision: RecoveryDispositionDecision::Retire,
    }
}

fn receipt_path(root: &Path) -> PathBuf {
    root.join("recovery")
        .join("dispositions")
        .join(format!("{ROOT_DIGEST_HEX}.receipt"))
}

fn root_pool_entry(root: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let root_bytes = fixture(ROOT_HEX)?;
    let admitted = AdmittedRetentionRoot::decode(&root_bytes)?;
    Ok(root
        .join("retention")
        .join("roots")
        .join(pool_name::namespace(admitted.root().namespace().digest()))
        .join(pool_name::root(
            admitted.root().generation(),
            admitted.digest(),
        )))
}

fn publish_initial(root: &Path) -> Result<RetentionPublicationOutcome, Box<dyn Error>> {
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    let mut authority = reopen_authority(root)?;
    Ok(execute_retention_publication(&mut authority, &preparation)?.outcome())
}

#[test]
fn retiring_a_protected_root_under_an_absent_head_frees_publication() -> Result<(), Box<dyn Error>>
{
    let sandbox = interrupted_store("disposition-retire-root", 7)?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    let mut authority = reopen_authority(sandbox.path())?;
    let error = authority
        .verify_current(&preparation)
        .err()
        .ok_or("a protected orphan was admitted for publication")?;
    assert!(matches!(
        refusal(&error),
        Some(RetentionCurrentStateRefusal::RetainedStage)
    ));

    let receipt = authority.dispose(retire(RecoveryDispositionTarget::Root))?;
    drop(authority);

    let semantic = receipt.receipt();
    assert_eq!(
        semantic.artifact().kind,
        RecoveryArtifactKind::RetentionRoot
    );
    assert_eq!(
        semantic.artifact().classification,
        RecoveryClassification::CompleteOrphan
    );
    assert_eq!(semantic.decision(), RecoveryDispositionDecision::Retire);
    assert_eq!(semantic.coordinates().retention, GcRetentionState::Empty);
    assert_eq!(semantic.coordinates().publication_generation.get(), 1);
    assert_eq!(
        semantic.artifact().identity_digest.as_bytes().as_slice(),
        crate::adapters::test_support::decode_hex(ROOT_DIGEST_HEX)?.as_slice()
    );
    let stored = fs::read(receipt_path(sandbox.path()))?;
    assert_eq!(stored, receipt.encoded());
    assert!(AdmittedRecoveryDispositionReceipt::decode(&stored).is_ok());
    assert!(!sandbox.path().join("retention").join("root.next").exists());
    assert!(
        !sandbox
            .path()
            .join("recovery")
            .join("disposition.next")
            .exists()
    );
    assert!(!root_pool_entry(sandbox.path())?.exists());
    assert_eq!(
        fs::read_dir(sandbox.path().join("retention").join("roots"))?.count(),
        0,
        "the emptied namespace directory is removed"
    );

    assert_eq!(
        publish_initial(sandbox.path())?,
        RetentionPublicationOutcome::Published
    );
    sandbox.remove()?;
    Ok(())
}

#[test]
fn finalize_requires_a_published_head() -> Result<(), Box<dyn Error>> {
    let sandbox = interrupted_store("disposition-finalize-absent-head", 7)?;
    let mut authority = reopen_authority(sandbox.path())?;

    let error = authority
        .dispose(request(
            RecoveryDispositionTarget::Root,
            RecoveryDispositionDecision::Finalize,
        ))
        .err()
        .ok_or("an orphan was finalized into no visible state")?;

    assert!(matches!(
        error,
        DispositionError::Plan {
            source: RecoveryDispositionRefusal::FinalizeRequiresPublishedHead
        }
    ));
    assert!(sandbox.path().join("retention").join("root.next").exists());
    sandbox.remove()?;
    Ok(())
}

#[test]
fn finalizing_a_successor_orphan_keeps_its_pool_entry_and_frees_publication()
-> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("disposition-finalize-successor")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let _published =
        execute_retention_publication(&mut authority, &initial_preparation(&root_bytes)?)?;
    let current_root = AdmittedRetentionRoot::decode(&root_bytes)?;
    let manifest_bytes = fixture(MANIFEST_HEX)?;
    let current_manifest = AdmittedRetentionManifest::decode(&manifest_bytes)?;
    let candidate = successor_root(&current_root)?;
    let preparation = successor_preparation(&current_root, &current_manifest, candidate.encoded())?;
    drive_publication(&mut authority, &preparation, 7)?;

    let receipt = authority.dispose(request(
        RecoveryDispositionTarget::Root,
        RecoveryDispositionDecision::Finalize,
    ))?;

    assert_eq!(
        receipt.receipt().decision(),
        RecoveryDispositionDecision::Finalize
    );
    assert!(matches!(
        receipt.receipt().coordinates().retention,
        GcRetentionState::Published { .. }
    ));
    assert!(!sandbox.path().join("retention").join("root.next").exists());
    let successor = AdmittedRetentionRoot::decode(candidate.encoded())?;
    let entry = sandbox
        .path()
        .join("retention")
        .join("roots")
        .join(pool_name::namespace(successor.root().namespace().digest()))
        .join(pool_name::root(
            successor.root().generation(),
            successor.digest(),
        ));
    assert!(entry.exists(), "finalize keeps the immutable pool entry");

    let other = initial_root(b"other", &current_root)?;
    let other_preparation = new_namespace_preparation(&current_manifest, other.encoded())?;
    let published = execute_retention_publication(&mut authority, &other_preparation)?;
    assert_eq!(published.outcome(), RetentionPublicationOutcome::Published);
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_second_disposition_finds_nothing_protected_and_changes_nothing() -> Result<(), Box<dyn Error>>
{
    let sandbox = interrupted_store("disposition-twice", 7)?;
    let mut authority = reopen_authority(sandbox.path())?;
    let receipt = authority.dispose(retire(RecoveryDispositionTarget::Root))?;

    let error = authority
        .dispose(retire(RecoveryDispositionTarget::Root))
        .err()
        .ok_or("a disposed store was disposed again")?;

    assert!(matches!(
        error,
        DispositionError::Plan {
            source: RecoveryDispositionRefusal::NothingProtected
        }
    ));
    assert_eq!(fs::read(receipt_path(sandbox.path()))?, receipt.encoded());
    sandbox.remove()?;
    Ok(())
}

#[test]
fn the_manifest_stage_must_be_disposed_before_the_root_it_names() -> Result<(), Box<dyn Error>> {
    let sandbox = interrupted_store("disposition-order", 11)?;
    let mut authority = reopen_authority(sandbox.path())?;

    let error = authority
        .dispose(retire(RecoveryDispositionTarget::Root))
        .err()
        .ok_or("the root was disposed under a protected manifest")?;
    assert!(matches!(
        error,
        DispositionError::Plan {
            source: RecoveryDispositionRefusal::ManifestStageRemains
        }
    ));

    let manifest = authority.dispose(retire(RecoveryDispositionTarget::Manifest))?;
    assert_eq!(
        manifest.receipt().artifact().kind,
        RecoveryArtifactKind::RetentionManifest
    );
    let root = authority.dispose(retire(RecoveryDispositionTarget::Root))?;
    assert_eq!(
        root.receipt().artifact().kind,
        RecoveryArtifactKind::RetentionRoot
    );
    drop(authority);
    assert_eq!(
        fs::read_dir(sandbox.path().join("retention").join("manifests"))?.count(),
        0
    );
    assert_eq!(
        publish_initial(sandbox.path())?,
        RetentionPublicationOutcome::Published
    );
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_reader_holding_the_fence_refuses_disposition_without_waiting() -> Result<(), Box<dyn Error>> {
    let sandbox = interrupted_store("disposition-readers", 7)?;
    let directory = Dir::open_ambient_dir(sandbox.path(), cap_std::ambient_authority())?;
    let fence = ReaderFence::acquire(&directory)?;
    let mut authority = reopen_authority(sandbox.path())?;

    let error = authority
        .dispose(retire(RecoveryDispositionTarget::Root))
        .err()
        .ok_or("disposition ran beside an active reader")?;

    assert!(matches!(error, DispositionError::ReadersActive));
    assert!(sandbox.path().join("retention").join("root.next").exists());
    drop(fence);
    let _receipt = authority.dispose(retire(RecoveryDispositionTarget::Root))?;
    sandbox.remove()?;
    Ok(())
}

/// Reconstructs each residue an interrupted retirement can leave and proves
/// the next call with the same request resumes to the same complete state,
/// or refuses a residue that names another decision.
#[test]
fn an_interrupted_retirement_resumes_from_every_residue() -> Result<(), Box<dyn Error>> {
    let sandbox = interrupted_store("disposition-resume", 7)?;
    let mut authority = reopen_authority(sandbox.path())?;
    let receipt = authority.dispose(retire(RecoveryDispositionTarget::Root))?;
    drop(authority);
    let root_stage = sandbox.path().join("retention").join("root.next");
    let stage = sandbox.path().join("recovery").join("disposition.next");
    let linked = receipt_path(sandbox.path());
    let pool_entry = root_pool_entry(sandbox.path())?;
    let root_bytes = fixture(ROOT_HEX)?;
    let expected = receipt.encoded().to_vec();
    let restore_orphan = || -> Result<(), Box<dyn Error>> {
        fs::create_dir_all(pool_entry.parent().ok_or("namespace")?)?;
        fs::write(&pool_entry, &root_bytes)?;
        fs::hard_link(&pool_entry, &root_stage)?;
        Ok(())
    };
    let settled = |label: &str| -> Result<(), Box<dyn Error>> {
        assert!(!stage.exists(), "{label}: stage");
        assert!(!root_stage.exists(), "{label}: root stage");
        assert!(!pool_entry.exists(), "{label}: pool entry");
        assert_eq!(fs::read(&linked)?, expected, "{label}: receipt");
        Ok(())
    };

    // Death after the receipt was linked, before stage removal.
    restore_orphan()?;
    fs::hard_link(&linked, &stage)?;
    let again =
        reopen_authority(sandbox.path())?.dispose(retire(RecoveryDispositionTarget::Root))?;
    assert_eq!(again.encoded(), expected.as_slice());
    settled("after link")?;

    // Death after the disposition stage was removed, before the retained
    // root stage was.
    restore_orphan()?;
    let _receipt =
        reopen_authority(sandbox.path())?.dispose(retire(RecoveryDispositionTarget::Root))?;
    settled("after stage removal")?;

    // Death after the retained stage was removed, before the pool entry was.
    fs::create_dir_all(pool_entry.parent().ok_or("namespace")?)?;
    fs::write(&pool_entry, &root_bytes)?;
    let _receipt =
        reopen_authority(sandbox.path())?.dispose(retire(RecoveryDispositionTarget::Root))?;
    settled("after retained stage removal")?;

    // Death mid-write of the disposition stage before any receipt: the
    // truncated stage is discarded and the disposition redone.
    fs::remove_file(&linked)?;
    restore_orphan()?;
    fs::write(&stage, expected.get(..100).ok_or("prefix")?)?;
    let _receipt =
        reopen_authority(sandbox.path())?.dispose(retire(RecoveryDispositionTarget::Root))?;
    settled("after truncated stage")?;

    // A stage or receipt naming another decision is ambiguity, not a retry.
    restore_orphan()?;
    let mut other = expected.clone();
    let byte = other.get_mut(27).ok_or("decision byte")?;
    *byte = 1;
    fs::write(&stage, &other)?;
    let error = reopen_authority(sandbox.path())?
        .dispose(retire(RecoveryDispositionTarget::Root))
        .err()
        .ok_or("a foreign stage was resumed")?;
    assert!(matches!(
        error,
        DispositionError::Ambiguity(RecoveryDispositionAmbiguity::StageDiffers)
    ));
    fs::remove_file(&stage)?;
    fs::write(&linked, &other)?;
    let error = reopen_authority(sandbox.path())?
        .dispose(retire(RecoveryDispositionTarget::Root))
        .err()
        .ok_or("a foreign receipt was resumed")?;
    assert!(matches!(
        error,
        DispositionError::Ambiguity(RecoveryDispositionAmbiguity::ReceiptDiffers)
    ));
    sandbox.remove()?;
    Ok(())
}

#[test]
fn readers_admit_a_store_with_receipts_and_refuse_a_stray_disposition_entry()
-> Result<(), Box<dyn Error>> {
    let sandbox = interrupted_store("disposition-admission", 7)?;
    let _receipt =
        reopen_authority(sandbox.path())?.dispose(retire(RecoveryDispositionTarget::Root))?;

    let view = FilesystemRetentionSnapshot::load(
        sandbox.path(),
        catalog_policy()?,
        ReaderAttemptLimit::DEFAULT,
    )?;
    drop(view);
    fs::write(
        sandbox
            .path()
            .join("recovery")
            .join("dispositions")
            .join("stray"),
        b"x",
    )?;
    assert!(
        FilesystemRetentionSnapshot::load(
            sandbox.path(),
            catalog_policy()?,
            ReaderAttemptLimit::DEFAULT
        )
        .is_err(),
        "a stray disposition entry was admitted"
    );
    sandbox.remove()?;
    Ok(())
}

#[path = "filesystem_retention_disposition_effect_tests.rs"]
mod effect_tests;
