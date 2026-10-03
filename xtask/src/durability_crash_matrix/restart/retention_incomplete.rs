//! This module owns evidence preservation after a process dies during a retention stage write.

use super::super::{
    DurabilityCrashMatrixError,
    production_protocol::{
        fixture::GoldenFixture,
        retention::{preparation, reopened_authority},
        verification,
    },
};
use keep::{
    FilesystemRetentionRecoveryError, RetentionCurrentStateRefusal, RetentionFixedStage as Stage,
    RetentionPublicationError, RetentionRecoveryRefusal, RetentionRecoveryStep as Step,
    execute_retention_publication,
};
use std::{collections::BTreeMap, fs, io, path::Path};

pub(super) fn verify(store: &Path, interrupted: Step) -> Result<(), DurabilityCrashMatrixError> {
    let (stage, expected) = match interrupted {
        Step::DiscardRootStage => (Stage::Root, 192),
        Step::DiscardManifestStage => (Stage::Manifest, 160),
        Step::DiscardHeadStage => (Stage::Head, 144),
        _ => return Err(failure("unexpected incomplete-stage coordinate")),
    };
    let before = witness(store)?;
    let mut authority = reopened_authority(store)?;
    let result = authority.recover();
    if !matches!(result, Err(FilesystemRetentionRecoveryError::Plan { source: RetentionRecoveryRefusal::IncompleteStageRequiresDisposition { stage: actual, expected: minimum, observed: 100 } }) if actual == stage && minimum == expected)
    {
        return Err(failure(&format!(
            "expected disposition for {stage:?}, got {result:?}"
        )));
    }
    if witness(store)? != before {
        return Err(failure(
            "incomplete-stage recovery changed retained evidence",
        ));
    }
    let root = GoldenFixture::retention_root()?;
    super::retention_snapshot::verify(store, root.bytes(), None)?;
    let preparation = preparation(root.bytes())?;
    let retry = execute_retention_publication(&mut authority, &preparation);
    if !matches!(retry, Err(RetentionPublicationError::CurrentVerification { ref source }) if matches!(source.get_ref().and_then(|cause| cause.downcast_ref::<RetentionCurrentStateRefusal>()), Some(RetentionCurrentStateRefusal::RecoveryRefused { source: RetentionRecoveryRefusal::IncompleteStageRequiresDisposition { stage: actual, expected: minimum, observed: 100 } }) if *actual == stage && *minimum == expected))
    {
        return Err(failure(&format!(
            "publication did not preserve disposition requirement: {retry:?}"
        )));
    }
    if witness(store)? != before {
        return Err(failure("publication retry changed retained evidence"));
    }
    Ok(())
}

fn witness(store: &Path) -> Result<BTreeMap<String, Option<Vec<u8>>>, DurabilityCrashMatrixError> {
    super::inventory(store)?
        .into_iter()
        .map(|name| {
            let path = store.join(&name);
            let metadata = fs::symlink_metadata(&path)
                .map_err(|source| verification("inspect retained crash evidence", source))?;
            let bytes = if metadata.is_file() {
                Some(
                    fs::read(path)
                        .map_err(|source| verification("read retained crash evidence", source))?,
                )
            } else {
                None
            };
            Ok((name, bytes))
        })
        .collect()
}

fn failure(message: &str) -> DurabilityCrashMatrixError {
    verification(
        "verify incomplete retention restart",
        io::Error::other(message.to_owned()),
    )
}
