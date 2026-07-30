//! Deterministic initialized-store fixture for filesystem stage completion.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use crate::LayoutEntryLimit;

use super::super::{
    FilesystemRecoveryStageCompleter, FilesystemRecoveryStageCompletionOpenError, RecoveryStage,
    RecoveryStageCompletionRequest, RecoveryStageCompletionTarget, RecoveryStageMetadata,
    SegmentReadPolicy, SegmentRecordLimit, admit_recovery_stage_bytes, assess_recovery_stage,
    filesystem_test_sandbox::TestDirectory, fingerprint_recovery_stage, physical_pool_name,
    plan_recovery_stage_completion, test_support::decode_hex,
};

const SEGMENT_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-segment.hex");
const CATALOG_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-catalog.hex");

pub(super) fn segment_bytes() -> Result<Vec<u8>, Box<dyn Error>> {
    fixture(SEGMENT_HEX)
}

pub(super) fn catalog_bytes() -> Result<Vec<u8>, Box<dyn Error>> {
    fixture(CATALOG_HEX)
}

pub(super) fn request(
    stage: RecoveryStage,
    bytes: &[u8],
) -> Result<RecoveryStageCompletionRequest, Box<dyn Error>> {
    let length = u64::try_from(bytes.len())?;
    let observed = fingerprint_recovery_stage(RecoveryStageMetadata::new(stage, length)?, bytes)?;
    let admitted = admit_recovery_stage_bytes(stage, observed, bytes)?;
    let assessed = assess_recovery_stage(&admitted, maximum_policy())?;
    Ok(plan_recovery_stage_completion(&assessed)?)
}

fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(
        hex.strip_suffix('\n')
            .ok_or("recovery fixture must end in one LF")?,
    )
    .map_err(Into::into)
}

const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}

pub(super) struct CompletionFixture {
    directory: TestDirectory,
}

impl CompletionFixture {
    pub(super) fn new(name: &str) -> Result<Self, Box<dyn Error>> {
        let directory = TestDirectory::create(name)?;
        fs::write(directory.path().join("writer.lock"), [])?;
        for name in ["staging", "segments", "catalogs"] {
            fs::create_dir(directory.path().join(name))?;
        }
        Ok(Self { directory })
    }

    pub(super) fn root(&self) -> &Path {
        self.directory.path()
    }

    pub(super) fn stage_path(&self, stage: RecoveryStage) -> PathBuf {
        match stage {
            RecoveryStage::Segment => self.root().join("staging/current.seg"),
            RecoveryStage::Catalog => self.root().join("staging/current.cat"),
            RecoveryStage::NextHead => self.root().join("head.next"),
        }
    }

    pub(super) fn pool_path(&self, request: RecoveryStageCompletionRequest) -> PathBuf {
        match request.target() {
            RecoveryStageCompletionTarget::Segment { digest } => self
                .root()
                .join("segments")
                .join(physical_pool_name::segment(digest)),
            RecoveryStageCompletionTarget::Catalog {
                generation, digest, ..
            } => self
                .root()
                .join("catalogs")
                .join(physical_pool_name::catalog(generation, digest)),
        }
    }

    pub(super) fn completer(
        &self,
    ) -> Result<FilesystemRecoveryStageCompleter, FilesystemRecoveryStageCompletionOpenError> {
        FilesystemRecoveryStageCompleter::open_unchecked_for_tests(self.root())
    }

    pub(super) fn remove(self) -> std::io::Result<()> {
        self.directory.remove()
    }
}
