//! This module owns disposition failures after observable namespace effects.

use super::{
    DispositionError, interrupted_store, receipt_path, reopen_authority, retire, root_pool_entry,
};
use crate::adapters::retention::{
    RecoveryDispositionPhase as Phase, RecoveryDispositionTarget,
    RetentionEffectDurability as Durability, RetentionNamespaceEffect as Effect,
    RetentionStorageBoundary as Boundary,
};
use std::{error::Error, fs, io};

// Size: medium. Oracle: refusal is not rollback; an actual successful unlink
// followed by failed directory synchronization remains a known removal with
// unconfirmed durability. EIO is a deterministic checkpoint fault, not power loss.
// Delete only if stronger public disposition failure laws subsume this outcome.
#[test]
fn failed_namespace_sync_reports_the_removed_pool_artifact() -> Result<(), Box<dyn Error>> {
    let sandbox = interrupted_store("disposition-pool-sync-effects", 7)?;
    let pool = root_pool_entry(sandbox.path())?;
    let published_head = fs::read(sandbox.path().join("HEAD"))?;
    let mut authority = reopen_authority(sandbox.path())?;
    authority.disposition_failure = Some(Boundary::NamespaceSynchronization);
    let Err(DispositionError::Execute { source }) =
        authority.dispose(retire(RecoveryDispositionTarget::Root))
    else {
        return Err("injected sync failure did not reach the public execution error".into());
    };
    assert_eq!(source.phase(), Phase::RemovePoolEntry);
    assert_eq!(
        fs::metadata(&pool).err().map(|e| e.kind()),
        Some(io::ErrorKind::NotFound),
        "unlink happened before synchronization failed"
    );
    assert!(
        fs::metadata(pool.parent().ok_or("namespace absent")?)?.is_dir(),
        "later namespace removal must not run"
    );
    let receipt = fs::read(receipt_path(sandbox.path()))?;
    let admitted = crate::adapters::AdmittedRecoveryDispositionReceipt::decode(&receipt)?;
    assert_eq!(
        admitted.receipt().decision(),
        crate::adapters::RecoveryDispositionDecision::Retire
    );
    assert_eq!(
        fs::read(sandbox.path().join("HEAD"))?,
        published_head,
        "published data authority survives"
    );
    assert_eq!(
        source.storage_progress().map(|p| p.boundary()),
        Some(Boundary::NamespaceSynchronization),
        "failed capability must report the exact post-unlink boundary"
    );
    let progress = source
        .storage_progress()
        .ok_or("missing storage progress")?;
    let effects: Vec<_> = progress
        .known_effects()
        .iter()
        .map(|e| (e.effect(), e.durability()))
        .collect();
    assert_eq!(
        effects,
        [(Effect::PoolEntryRemoved, Durability::Unconfirmed)]
    );
    assert_eq!(
        progress.uncertain_effect(),
        None,
        "successful unlink is known, not uncertain"
    );
    Ok(())
}
