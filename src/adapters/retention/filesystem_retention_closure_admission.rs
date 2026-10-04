//! This module owns capability-relative live closure admission under retention authority.

use std::io;

use cap_std::fs::Dir;

use super::{
    RetentionRecoveryEvidence, RetentionStageAssessment, VerifiedRetentionClosure,
    verify_retention_closure,
};
use crate::RetentionRoot;
use crate::adapters::{CatalogRestartPolicy, catalog_restart_loader};

pub(super) fn admit_recovery(
    root: &Dir,
    evidence: &RetentionRecoveryEvidence<'_, '_>,
    policy: CatalogRestartPolicy,
) -> io::Result<()> {
    let RetentionStageAssessment::Complete(candidate) = &evidence.stages().root else {
        return Ok(());
    };
    verify(root, candidate.root(), policy).map(|_verified| ())
}

pub(super) fn verify(
    root: &Dir,
    candidate: &RetentionRoot,
    policy: CatalogRestartPolicy,
) -> io::Result<VerifiedRetentionClosure> {
    let loaded = catalog_restart_loader::load_from_directory(root, "HEAD", policy)
        .map_err(|source| io::Error::new(io::ErrorKind::InvalidData, source))?;
    let snapshot = loaded
        .snapshot()
        .map_err(|source| io::Error::new(io::ErrorKind::InvalidData, source))?;
    verify_retention_closure(candidate, &snapshot)
        .map_err(|source| io::Error::new(io::ErrorKind::InvalidData, source))
}
