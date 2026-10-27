//! Deterministic initialized-store fixture for filesystem segment continuation.

use std::error::Error;
use std::path::{Path, PathBuf};

use crate::LayoutEntryLimit;

use super::super::{
    FilesystemPlatformAdmissionError, FilesystemRecoverySegmentResumer,
    FilesystemVersionTwoAdmission, FilesystemVersionTwoRecoveryAdmission,
    RecoverySegmentResumeRequest, RecoveryStage, RecoveryStageMetadata, SegmentReadPolicy,
    SegmentRecordLimit, admit_recovery_stage_bytes, assess_recovery_stage,
    filesystem_test_sandbox::TestDirectory, fingerprint_recovery_stage,
    plan_recovery_segment_resume, test_support::decode_hex,
};

const SEGMENT_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-segment.hex");
const SEGMENT_SEAL_LENGTH: usize = 128;
const SEGMENT_HEADER_LENGTH: usize = 64;

pub(super) fn reusable_prefix() -> Result<Vec<u8>, Box<dyn Error>> {
    let mut encoded = decode_hex(
        SEGMENT_HEX
            .strip_suffix('\n')
            .ok_or("recovery fixture must end in one LF")?,
    )?;
    let length = encoded
        .len()
        .checked_sub(SEGMENT_SEAL_LENGTH)
        .ok_or("segment fixture is shorter than its seal")?;
    encoded.truncate(length);
    Ok(encoded)
}

pub(super) fn empty_prefix() -> Result<Vec<u8>, Box<dyn Error>> {
    let mut encoded = reusable_prefix()?;
    encoded.truncate(SEGMENT_HEADER_LENGTH);
    Ok(encoded)
}

pub(super) fn resume_request(bytes: &[u8]) -> Result<RecoverySegmentResumeRequest, Box<dyn Error>> {
    let length = u64::try_from(bytes.len())?;
    let observed = fingerprint_recovery_stage(
        RecoveryStageMetadata::new(RecoveryStage::Segment, length)?,
        bytes,
    )?;
    let admitted = admit_recovery_stage_bytes(RecoveryStage::Segment, observed, bytes)?;
    let assessed = assess_recovery_stage(&admitted, maximum_policy())?;
    Ok(plan_recovery_segment_resume(&assessed, maximum_policy())?)
}

pub(super) const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}

pub(super) struct ResumeFixture {
    directory: TestDirectory,
}

impl ResumeFixture {
    pub(super) fn new(name: &str) -> Result<Self, Box<dyn Error>> {
        let directory = TestDirectory::create(name)?;
        drop(
            FilesystemVersionTwoAdmission::initialize_unchecked_for_repository_tasks(
                directory.path(),
            )?,
        );
        Ok(Self { directory })
    }

    pub(super) fn root(&self) -> &Path {
        self.directory.path()
    }

    pub(super) fn stage_path(&self) -> PathBuf {
        self.root().join("staging/current.seg")
    }

    /// Acquires single-format recovery admission, the writer authority continuation runs under.
    pub(super) fn recovery_admission(
        &self,
    ) -> Result<FilesystemVersionTwoRecoveryAdmission, FilesystemPlatformAdmissionError> {
        FilesystemVersionTwoRecoveryAdmission::reopen_unchecked_for_repository_tasks(self.root())
    }

    pub(super) fn resumer(&self) -> Result<FilesystemRecoverySegmentResumer, Box<dyn Error>> {
        Ok(FilesystemRecoverySegmentResumer::open_version_two(
            self.recovery_admission()?,
        )?)
    }

    pub(super) fn resumer_before_handoff<F>(
        &self,
        before_handoff: F,
    ) -> Result<FilesystemRecoverySegmentResumer, Box<dyn Error>>
    where
        F: FnOnce() + 'static,
    {
        Ok(
            FilesystemRecoverySegmentResumer::open_before_handoff_for_tests(
                self.recovery_admission()?,
                before_handoff,
            )?,
        )
    }

    pub(super) fn remove(self) -> std::io::Result<()> {
        self.directory.remove()
    }
}
