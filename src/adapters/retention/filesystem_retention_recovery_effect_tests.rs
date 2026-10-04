//! This module owns filesystem effects reported after recovery synchronization failures.

use crate::adapters::FilesystemVersionTwoAdmission;
use crate::adapters::retention::filesystem_retention_test_fixture::{
    ROOT_HEX, drive_publication, fixture, initial_preparation, manifest_pool_path, open_authority,
    retention_witness, root_pool_path,
};
use crate::adapters::retention::{
    FilesystemRetentionPublicationAuthority, FilesystemRetentionRecoveryError,
    RetentionEffectDurability as Durability, RetentionNamespaceEffect as Effect,
    RetentionRecoveryStep as Step, RetentionStorageBoundary as Boundary,
};
use std::{error::Error, fs, io};

struct Case {
    phases: usize,
    boundary: Boundary,
    step: Step,
    effects: &'static [(Effect, Durability)],
}

// Size: medium. Oracle: successful namespace calls remain visible after an injected sync failure.
// Actual filesystem operations with deterministic pre-sync EIO; no claim of physical power loss.
// Delete only when stronger public failure/restart coverage subsumes each capability below.
#[test]
fn recovery_sync_failures_report_exact_known_effects_before_restart() -> Result<(), Box<dyn Error>>
{
    for case in [
        Case {
            phases: 2,
            boundary: Boundary::RootsSynchronization,
            step: Step::LinkRoot,
            effects: &[(Effect::NamespaceCreated, Durability::Unconfirmed)],
        },
        Case {
            phases: 2,
            boundary: Boundary::PoolSynchronization,
            step: Step::LinkRoot,
            effects: &[
                (Effect::NamespaceCreated, Durability::Synchronized),
                (Effect::PoolLinkCreated, Durability::Unconfirmed),
            ],
        },
        Case {
            phases: 8,
            boundary: Boundary::PoolSynchronization,
            step: Step::LinkManifest,
            effects: &[(Effect::PoolLinkCreated, Durability::Unconfirmed)],
        },
        Case {
            phases: 13,
            boundary: Boundary::RetentionSynchronization,
            step: Step::FinalizeHead,
            effects: &[(Effect::HeadReplaced, Durability::Unconfirmed)],
        },
        Case {
            phases: 16,
            boundary: Boundary::RetentionSynchronization,
            step: Step::RemoveManifestStage,
            effects: &[(Effect::StageRemoved, Durability::Unconfirmed)],
        },
    ] {
        require_effects(&case)?;
    }
    Ok(())
}

fn require_effects(case: &Case) -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority(&format!(
        "recovery-effect-{:?}-{:?}",
        case.step, case.boundary
    ))?;
    let root = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root)?;
    drive_publication(&mut authority, &preparation, case.phases)?;
    let mut expected = retention_witness(sandbox.path())?;
    authority.recovery_sync_failure = Some(case.boundary);
    let error = match authority.recover() {
        Err(FilesystemRetentionRecoveryError::Execute { source }) => source,
        result => return Err(format!("expected sync failure: {result:?}").into()),
    };
    require_progress(&error, case)?;
    apply_expected_effects(case, sandbox.path(), &preparation, &root, &mut expected)?;
    assert_eq!(
        retention_witness(sandbox.path())?,
        expected,
        "only reported effects may change retained evidence"
    );
    drop(authority);
    let mut restarted = FilesystemRetentionPublicationAuthority::open(
        FilesystemVersionTwoAdmission::reopen_unchecked_for_repository_tasks(sandbox.path())?,
        crate::adapters::retention::filesystem_retention_test_fixture::catalog_policy()?,
    )?;
    let receipt = restarted.recover()?;
    assert_eq!(
        receipt.outcome(),
        match case.step {
            Step::LinkRoot => crate::RetentionRecoveryOutcome::Protected {
                root_stage: true,
                manifest_stage: false
            },
            Step::LinkManifest => crate::RetentionRecoveryOutcome::Protected {
                root_stage: true,
                manifest_stage: true
            },
            Step::FinalizeHead => crate::RetentionRecoveryOutcome::Committed,
            _ => crate::RetentionRecoveryOutcome::Clean,
        }
    );
    assert_eq!(
        fs::read(root_pool_path(sandbox.path(), preparation.candidate()))?,
        root,
        "restart retains exact root evidence"
    );
    Ok(())
}

fn require_progress(
    error: &crate::RetentionRecoveryError,
    case: &Case,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(error.step(), case.step);
    assert!(
        error.executed().is_empty(),
        "effects belong to the first failing capability"
    );
    let progress = error.progress().ok_or("missing effects")?;
    assert_eq!(progress.boundary(), case.boundary);
    assert_eq!(
        progress
            .known_effects()
            .iter()
            .map(|effect| (effect.effect(), effect.durability()))
            .collect::<Vec<_>>(),
        case.effects
    );
    assert_eq!(progress.uncertain_effect(), None);
    assert_eq!(
        io_cause(error).and_then(io::Error::raw_os_error),
        Some(rustix::io::Errno::IO.raw_os_error())
    );
    Ok(())
}

fn apply_expected_effects(
    case: &Case,
    store: &std::path::Path,
    preparation: &crate::RetentionPublicationPreparation<'_>,
    root: &[u8],
    expected: &mut std::collections::BTreeSet<(std::ffi::OsString, Vec<u8>)>,
) -> Result<(), Box<dyn Error>> {
    let publication = preparation.publication().ok_or("publication absent")?;
    for &(effect, _) in case.effects {
        match effect {
            Effect::StageCreated
            | Effect::ReceiptReplaced
            | Effect::CandidateRemoved
            | Effect::IntentRemoved => {
                return Err("effect is outside this retention recovery fixture".into());
            }
            Effect::NamespaceCreated => assert!(
                root_pool_path(store, preparation.candidate())
                    .parent()
                    .ok_or("pool parent absent")?
                    .is_dir(),
                "reported namespace creation must be visible"
            ),
            Effect::PoolLinkCreated => {
                let (path, bytes) = if case.step == Step::LinkRoot {
                    (
                        root_pool_path(store, preparation.candidate()),
                        root.to_vec(),
                    )
                } else {
                    (
                        manifest_pool_path(store, preparation),
                        publication.manifest().encoded().to_vec(),
                    )
                };
                expected.insert((path.into_os_string(), bytes));
            }
            Effect::HeadReplaced => {
                expected.remove(&(
                    store.join("retention/head.next").into_os_string(),
                    publication.head().encoded().to_vec(),
                ));
                expected.insert((
                    store.join("retention/HEAD").into_os_string(),
                    publication.head().encoded().to_vec(),
                ));
            }
            Effect::StageRemoved => {
                expected.remove(&(
                    store.join("retention/manifest.next").into_os_string(),
                    publication.manifest().encoded().to_vec(),
                ));
            }
        }
    }
    Ok(())
}

fn io_cause<'a>(mut error: &'a (dyn Error + 'static)) -> Option<&'a io::Error> {
    loop {
        if let Some(source) = error.downcast_ref::<io::Error>() {
            return Some(source);
        }
        error = error.source()?;
    }
}
