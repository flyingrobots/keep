//! Deterministic initialized-store fixture for filesystem head finalization.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use crate::LayoutEntryLimit;

use super::super::{
    AdmittedSegment, CatalogPublicationExpectation, CatalogRestartByteLimit, CatalogRestartPolicy,
    CatalogSnapshot, ChecksummedCatalog, ChecksummedPublicationHead,
    FilesystemRecoveryNextHeadFinalizer, RecoveryNextHeadFinalizationRequest, RecoveryStage,
    RecoveryStageMetadata, SegmentReadPolicy, SegmentRecordLimit, admit_recovery_stage_bytes,
    assess_recovery_stage, filesystem_test_sandbox::TestDirectory, fingerprint_recovery_stage,
    physical_pool_name, plan_recovery_next_head_finalization, test_support::decode_hex,
};

const SEGMENT_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-segment.hex");
const CATALOG_ONE_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-catalog.hex");
const HEAD_ONE_HEX: &str = include_str!("../../../conformance/segment-store/v1/one-zero-head.hex");
const CATALOG_TWO_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-catalog-generation-two.hex");
const HEAD_TWO_HEX: &str =
    include_str!("../../../conformance/segment-store/v1/one-zero-head-generation-two.hex");
const RETAINED_SEGMENT_LIMIT: u64 = 1_048_576;

pub(super) struct FinalizationFixture {
    directory: TestDirectory,
}

impl FinalizationFixture {
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

    pub(super) fn head_path(&self) -> PathBuf {
        self.root().join("HEAD")
    }

    pub(super) fn next_head_path(&self) -> PathBuf {
        self.root().join("head.next")
    }

    pub(super) fn head_one() -> Result<Vec<u8>, Box<dyn Error>> {
        fixture(HEAD_ONE_HEX)
    }

    pub(super) fn head_two() -> Result<Vec<u8>, Box<dyn Error>> {
        fixture(HEAD_TWO_HEX)
    }

    pub(super) fn catalog_one_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        self.catalog_path(&fixture(CATALOG_ONE_HEX)?)
    }

    pub(super) fn install_generation_one_candidate(
        &self,
    ) -> Result<RecoveryNextHeadFinalizationRequest, Box<dyn Error>> {
        let segment = fixture(SEGMENT_HEX)?;
        let catalog = fixture(CATALOG_ONE_HEX)?;
        let head = Self::head_one()?;
        self.install_pool(&catalog, &segment)?;
        fs::write(self.next_head_path(), &head)?;
        request(
            &head,
            &catalog,
            &segment,
            CatalogPublicationExpectation::uninitialized(),
        )
    }

    pub(super) fn install_generation_two_candidate(
        &self,
    ) -> Result<RecoveryNextHeadFinalizationRequest, Box<dyn Error>> {
        let segment = fixture(SEGMENT_HEX)?;
        let catalog_one = fixture(CATALOG_ONE_HEX)?;
        let head_one = Self::head_one()?;
        let catalog_two = fixture(CATALOG_TWO_HEX)?;
        let head_two = Self::head_two()?;
        self.install_pool(&catalog_one, &segment)?;
        self.install_pool(&catalog_two, &segment)?;
        fs::write(self.head_path(), &head_one)?;
        fs::write(self.next_head_path(), &head_two)?;
        let current = snapshot(&head_one, &catalog_one, &segment)?;
        request(
            &head_two,
            &catalog_two,
            &segment,
            CatalogPublicationExpectation::successor_of(&current),
        )
    }

    pub(super) fn finalizer(&self) -> Result<FilesystemRecoveryNextHeadFinalizer, Box<dyn Error>> {
        Ok(FilesystemRecoveryNextHeadFinalizer::open_unchecked_for_tests(self.root(), policy()?)?)
    }

    pub(super) fn remove(self) -> std::io::Result<()> {
        self.directory.remove()
    }

    fn install_pool(&self, catalog: &[u8], segment: &[u8]) -> Result<(), Box<dyn Error>> {
        let admitted_segment = AdmittedSegment::decode(segment, maximum_segment_policy())?;
        let checksummed_catalog = ChecksummedCatalog::decode(catalog)?;
        let segment_name = physical_pool_name::segment(admitted_segment.digest());
        let catalog_name = physical_pool_name::catalog(
            checksummed_catalog.generation(),
            checksummed_catalog.digest(),
        );
        fs::write(self.root().join("segments").join(segment_name), segment)?;
        fs::write(self.root().join("catalogs").join(catalog_name), catalog)?;
        Ok(())
    }

    fn catalog_path(&self, bytes: &[u8]) -> Result<PathBuf, Box<dyn Error>> {
        let catalog = ChecksummedCatalog::decode(bytes)?;
        Ok(self
            .root()
            .join("catalogs")
            .join(physical_pool_name::catalog(
                catalog.generation(),
                catalog.digest(),
            )))
    }
}

fn request(
    head: &[u8],
    catalog: &[u8],
    segment: &[u8],
    expectation: CatalogPublicationExpectation,
) -> Result<RecoveryNextHeadFinalizationRequest, Box<dyn Error>> {
    let length = u64::try_from(head.len())?;
    let evidence = fingerprint_recovery_stage(
        RecoveryStageMetadata::new(RecoveryStage::NextHead, length)?,
        head,
    )?;
    let admitted = admit_recovery_stage_bytes(RecoveryStage::NextHead, evidence, head)?;
    let assessment = assess_recovery_stage(&admitted, maximum_segment_policy())?;
    let candidate = snapshot(head, catalog, segment)?;
    Ok(plan_recovery_next_head_finalization(
        &assessment,
        &candidate,
        expectation,
    )?)
}

fn snapshot<'bytes>(
    head: &'bytes [u8],
    catalog: &'bytes [u8],
    segment: &'bytes [u8],
) -> Result<CatalogSnapshot<'bytes, 'bytes, 'bytes>, Box<dyn Error>> {
    let segments = [AdmittedSegment::decode(segment, maximum_segment_policy())?];
    let admitted_catalog = ChecksummedCatalog::decode(catalog)?.admit(&segments)?;
    Ok(ChecksummedPublicationHead::decode(head)?.admit(admitted_catalog)?)
}

fn policy() -> Result<CatalogRestartPolicy, Box<dyn Error>> {
    Ok(CatalogRestartPolicy::new(
        maximum_segment_policy(),
        CatalogRestartByteLimit::new(RETAINED_SEGMENT_LIMIT)?,
    ))
}

const fn maximum_segment_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}

fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(
        hex.strip_suffix('\n')
            .ok_or("recovery fixture must end in one LF")?,
    )
    .map_err(Into::into)
}
