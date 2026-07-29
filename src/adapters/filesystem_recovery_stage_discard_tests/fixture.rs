//! Deterministic initialized-store fixture for filesystem stage discard.

use std::error::Error;
use std::fs;

use crate::LayoutEntryLimit;

use super::super::{
    FilesystemRecoveryStageDiscardOpenError, FilesystemRecoveryStageDiscarder, RecoveryStage,
    RecoveryStageDiscardRequest, RecoveryStageEvidence, RecoveryStageMetadata, SegmentReadPolicy,
    SegmentRecordLimit, admit_recovery_stage_bytes, assess_recovery_stage,
    filesystem_test_sandbox::TestDirectory, fingerprint_recovery_stage,
    plan_recovery_stage_discard,
};

pub(super) fn request(
    stage: RecoveryStage,
    bytes: &[u8],
) -> Result<RecoveryStageDiscardRequest, Box<dyn Error>> {
    let observed = evidence(stage, bytes)?;
    let admitted = admit_recovery_stage_bytes(stage, observed, bytes)?;
    let assessed = assess_recovery_stage(&admitted, maximum_policy())?;
    Ok(plan_recovery_stage_discard(&assessed)?)
}

pub(super) fn evidence(
    stage: RecoveryStage,
    bytes: &[u8],
) -> Result<RecoveryStageEvidence, Box<dyn Error>> {
    let length = u64::try_from(bytes.len())?;
    Ok(fingerprint_recovery_stage(
        RecoveryStageMetadata::new(stage, length)?,
        bytes,
    )?)
}

const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}

pub(super) struct DiscardFixture {
    directory: TestDirectory,
}

impl DiscardFixture {
    pub(super) fn new(name: &str) -> Result<Self, Box<dyn Error>> {
        let directory = TestDirectory::create(name)?;
        fs::write(directory.path().join("writer.lock"), [])?;
        for name in ["staging", "segments", "catalogs"] {
            fs::create_dir(directory.path().join(name))?;
        }
        Ok(Self { directory })
    }

    pub(super) fn root(&self) -> &std::path::Path {
        self.directory.path()
    }

    pub(super) fn stage_path(&self, stage: RecoveryStage) -> std::path::PathBuf {
        match stage {
            RecoveryStage::Segment => self.root().join("staging/current.seg"),
            RecoveryStage::Catalog => self.root().join("staging/current.cat"),
            RecoveryStage::NextHead => self.root().join("head.next"),
        }
    }

    pub(super) fn discarder(
        &self,
    ) -> Result<FilesystemRecoveryStageDiscarder, FilesystemRecoveryStageDiscardOpenError> {
        FilesystemRecoveryStageDiscarder::open_unchecked_for_tests(self.root())
    }

    pub(super) fn remove(self) -> std::io::Result<()> {
        self.directory.remove()
    }
}
