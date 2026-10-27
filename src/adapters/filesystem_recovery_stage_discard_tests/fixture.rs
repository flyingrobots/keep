//! Deterministic initialized-store fixture for filesystem stage discard.

use std::error::Error;

use crate::LayoutEntryLimit;

use super::super::{
    FilesystemRecoveryAdmission, FilesystemRecoveryStageDiscarder, FilesystemStoreAdmission,
    FilesystemStoreAdmissionError, RecoveryStage, RecoveryStageDiscardRequest,
    RecoveryStageEvidence, RecoveryStageMetadata, SegmentReadPolicy, SegmentRecordLimit,
    admit_recovery_stage_bytes, assess_recovery_stage, catalog_decoder,
    filesystem_test_sandbox::TestDirectory, fingerprint_recovery_stage,
    plan_recovery_stage_discard, publication_head_decoder, segment_header,
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

pub(super) fn truncated_bytes(
    stage: RecoveryStage,
    length: usize,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let framing: &[u8] = match stage {
        RecoveryStage::Segment => &segment_header::MAGIC,
        RecoveryStage::Catalog => &catalog_decoder::MAGIC,
        RecoveryStage::NextHead => &publication_head_decoder::MAGIC,
    };
    Ok(framing
        .get(..length)
        .ok_or("truncated fixture length exceeds fixed framing")?
        .to_vec())
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
        drop(
            FilesystemStoreAdmission::initialize_unchecked_for_repository_tasks(directory.path())?,
        );
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

    /// Acquires single-format recovery admission, the writer authority discard runs under.
    pub(super) fn recovery_admission(
        &self,
    ) -> Result<FilesystemRecoveryAdmission, FilesystemStoreAdmissionError> {
        FilesystemRecoveryAdmission::reopen_unchecked_for_repository_tasks(self.root())
    }

    pub(super) fn discarder(&self) -> Result<FilesystemRecoveryStageDiscarder, Box<dyn Error>> {
        Ok(FilesystemRecoveryStageDiscarder::open(
            self.recovery_admission()?,
        )?)
    }

    pub(super) fn remove(self) -> std::io::Result<()> {
        self.directory.remove()
    }
}
