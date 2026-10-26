//! These laws own runtime recovery of typed retention refusals from storage errors.
//! Size: medium (owned filesystem). Oracle: a caller recovers the exact refusal
//! and its original cause by downcasting, never by parsing a message.
//! Delete only when these storage ports stop returning `io::Error`.

use std::error::Error;
use std::fs;
use std::io;

use cap_std::ambient_authority;
use cap_std::fs::Dir;

use super::filesystem_retention_attempt::{self as attempt, PublicationAttempt};
use super::filesystem_retention_current::{verify_committed, verify_predecessor};
use super::filesystem_retention_recovery_observation::RetentionRecoveryObservation;
use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, initial_preparation, migrated_store, open_authority, refusal,
    successor_root,
};
use super::{
    AdmittedRetentionRoot, RetentionCurrentStateRefusal, RetentionFixedStage,
    RetentionRecordRefusal, RetentionRecoveryStorage, RetentionStorageBoundary,
    RetentionStorageError,
};
use crate::{LivenessGeneration, RetentionGenerationExpectation, execute_retention_publication};

fn record_refusal<T>(result: io::Result<T>) -> Option<(io::ErrorKind, RetentionRecordRefusal)> {
    let error = result.err()?;
    let refusal = error
        .get_ref()
        .and_then(|source| source.downcast_ref::<RetentionRecordRefusal>())
        .copied()?;
    Some((error.kind(), refusal))
}

const fn invalid(refusal: RetentionRecordRefusal) -> Option<(io::ErrorKind, RetentionRecordRefusal)> {
    Some((io::ErrorKind::InvalidData, refusal))
}

#[test]
fn publication_phases_without_retained_attempt_state_refuse_with_typed_payloads() {
    let fresh = PublicationAttempt::new(
        RetentionGenerationExpectation::Absent,
        LivenessGeneration::INITIAL,
    );
    let not_retained = |stage| RetentionRecordRefusal::StageNotRetained { stage };
    let no_pool = |stage| RetentionRecordRefusal::PoolCoordinateNotRetained { stage };

    assert_eq!(
        record_refusal(attempt::require(None)),
        invalid(RetentionRecordRefusal::PublicationAttemptAbsent)
    );
    assert_eq!(
        record_refusal(fresh.namespace()),
        invalid(RetentionRecordRefusal::NamespaceNotAdmitted)
    );
    assert_eq!(
        record_refusal(fresh.require_namespace("unadmitted")),
        invalid(RetentionRecordRefusal::NamespaceNotAdmitted)
    );
    assert_eq!(
        record_refusal(fresh.retained_root_name()),
        invalid(no_pool(RetentionFixedStage::Root))
    );
    assert_eq!(
        record_refusal(fresh.retained_manifest_name()),
        invalid(no_pool(RetentionFixedStage::Manifest))
    );
    assert_eq!(
        record_refusal(fresh.root_stage()),
        invalid(not_retained(RetentionFixedStage::Root))
    );
    assert_eq!(
        record_refusal(fresh.manifest_stage()),
        invalid(not_retained(RetentionFixedStage::Manifest))
    );
    assert_eq!(
        record_refusal(fresh.head_stage()),
        invalid(not_retained(RetentionFixedStage::Head))
    );
}

#[test]
fn recovery_steps_without_a_context_refuse_as_typed_record_state() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("retention-refusal-source-no-recovery")?;

    let error = RetentionRecoveryStorage::link_root(&mut authority)
        .err()
        .ok_or("a recovery step ran without a recovery context")?;

    let RetentionStorageError::Operation { source, progress } = &error else {
        return Err(format!("recovery step lost its boundary: {error:?}").into());
    };
    assert_eq!(progress.boundary(), RetentionStorageBoundary::RecoveryContext);
    assert!(
        matches!(
            source.as_ref(),
            RetentionStorageError::Refused {
                source: RetentionRecordRefusal::RecoveryAbsent
            }
        ),
        "recovery absence must be a typed record refusal: {error:?}"
    );
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_non_regular_retained_stage_refuses_observation_by_kind() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("retention-refusal-source-directory-stage")?;
    let retention_path = sandbox.path().join("retention");
    fs::create_dir(retention_path.join("root.next"))?;
    let retention = Dir::open_ambient_dir(&retention_path, ambient_authority())?;
    let roots = retention.open_dir("roots")?;
    let manifests = retention.open_dir("manifests")?;

    let observed = RetentionRecoveryObservation::observe(&retention, &roots, &manifests);

    assert_eq!(
        record_refusal(observed),
        invalid(RetentionRecordRefusal::KindOrLength)
    );
    sandbox.remove()?;
    Ok(())
}

#[test]
fn an_unopenable_selected_namespace_retains_its_original_cause() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("retention-refusal-source-namespace")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let _published =
        execute_retention_publication(&mut authority, &initial_preparation(&root_bytes)?)?;
    let current = authority
        .observe_current()?
        .ok_or("published retention head was not observed")?;
    let committed = AdmittedRetentionRoot::decode(&root_bytes)?;
    let successor_bytes = successor_root(&committed)?;
    let successor = AdmittedRetentionRoot::decode(successor_bytes.encoded())?;
    let empty_path = sandbox.path().join("empty-roots");
    fs::create_dir(&empty_path)?;
    let empty_roots = Dir::open_ambient_dir(&empty_path, ambient_authority())?;
    let namespace = committed.root().namespace().digest();

    for error in [
        verify_committed(&empty_roots, &current, &committed).err(),
        verify_predecessor(&empty_roots, &current, &successor).err(),
    ] {
        let error = error.ok_or("a missing selected namespace was admitted")?;
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(
            matches!(
                refusal(&error),
                Some(RetentionCurrentStateRefusal::CommittedNamespaceRead { namespace: observed, source })
                    if *observed == namespace && source.kind() == io::ErrorKind::NotFound
            ),
            "the selected namespace open failure must remain the source: {error:?}"
        );
    }
    drop(authority);
    sandbox.remove()?;
    Ok(())
}
