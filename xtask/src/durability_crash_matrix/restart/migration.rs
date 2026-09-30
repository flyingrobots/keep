//! This module owns independent post-process-death migration verification:
//! the exact residue, the predicted recovery plan, the production recovery,
//! and the complete migration it must leave behind.

use std::fs;
use std::path::Path;

use keep::{
    FilesystemStoreMigrationAuthority, FilesystemWriterLock, StoreMigrationRecoveryPlan as Plan,
    execute_store_migration, recover_store_migration,
};
use xtask::DurabilityCrashCase;

use super::migration_expectation::{MigrationExpectation, complete_paths};
use crate::durability_crash_matrix::DurabilityCrashMatrixError;
use crate::durability_crash_matrix::production_protocol::fixture::{
    CATALOG_POOL_PATH, GoldenFixture, SEGMENT_POOL_PATH,
};
use crate::durability_crash_matrix::production_protocol::initialization::segment_policy;
use crate::durability_crash_matrix::production_protocol::verification;

pub(super) fn verify(
    store_root: &Path,
    case: DurabilityCrashCase,
) -> Result<(), DurabilityCrashMatrixError> {
    let expected = MigrationExpectation::for_case(case)?;
    require_inventory(store_root, expected.paths())?;
    verify_version_one_bytes(store_root)?;

    let plan = recover(store_root)?;
    if plan != expected.plan() {
        return Err(DurabilityCrashMatrixError::RecoveryPlanMismatch {
            expected: expected.plan(),
            observed: plan,
        });
    }
    if plan == Plan::VersionOne {
        migrate_forward(store_root)?;
    }

    require_inventory(store_root, &complete_paths())?;
    verify_version_one_bytes(store_root)?;
    let settled = recover(store_root)?;
    if settled == Plan::Complete {
        Ok(())
    } else {
        Err(DurabilityCrashMatrixError::RecoveryPlanMismatch {
            expected: Plan::Complete,
            observed: settled,
        })
    }
}

fn require_inventory(
    store_root: &Path,
    expected: &std::collections::BTreeSet<String>,
) -> Result<(), DurabilityCrashMatrixError> {
    let observed = super::inventory(store_root)?;
    if &observed == expected {
        Ok(())
    } else {
        Err(DurabilityCrashMatrixError::InventoryMismatch {
            expected: expected.clone(),
            observed,
        })
    }
}

/// Reacquires writer authority the way a restarted process would and runs
/// the production recovery once, reporting the plan the residue admitted.
fn recover(store_root: &Path) -> Result<Plan, DurabilityCrashMatrixError> {
    let mut authority =
        FilesystemStoreMigrationAuthority::reopen_for_recovery_unchecked_for_repository_tasks(
            store_root,
            segment_policy(),
        )
        .map_err(|source| verification("reopen migration for recovery", source))?;
    let expected = authority
        .observe_intent()
        .map_err(|source| verification("observe recovery migration intent", source))?;
    let receipt = recover_store_migration(&mut authority, &expected)
        .map_err(|source| verification("recover production migration", source))?;
    Ok(receipt.plan())
}

/// The forward retry after an untouched version-1 store admits.
fn migrate_forward(store_root: &Path) -> Result<(), DurabilityCrashMatrixError> {
    let lock = FilesystemWriterLock::try_acquire(store_root)
        .map_err(|source| verification("reacquire writer lock for forward retry", source))?;
    let mut authority = FilesystemStoreMigrationAuthority::open_unchecked_for_repository_tasks(
        lock,
        segment_policy(),
    )
    .map_err(|source| verification("open forward-retry migration authority", source))?;
    let intent = authority
        .observe_intent()
        .map_err(|source| verification("observe forward-retry migration intent", source))?;
    execute_store_migration(&mut authority, &intent)
        .map(|_receipt| ())
        .map_err(|source| verification("execute forward-retry migration", source))
}

/// Migration never rewrites a version-1 byte: the pools and `HEAD` remain
/// the Golden File Worldline fixtures before and after recovery.
fn verify_version_one_bytes(store_root: &Path) -> Result<(), DurabilityCrashMatrixError> {
    for (relative, fixture) in [
        (SEGMENT_POOL_PATH, GoldenFixture::segment()?),
        (CATALOG_POOL_PATH, GoldenFixture::catalog()?),
        ("HEAD", GoldenFixture::head()?),
    ] {
        let observed = fs::read(store_root.join(relative))
            .map_err(|source| DurabilityCrashMatrixError::io("read version-1 artifact", source))?;
        if observed != fixture.bytes() {
            return Err(DurabilityCrashMatrixError::artifact_bytes(
                relative,
                fixture.bytes(),
                &observed,
            ));
        }
    }
    Ok(())
}
