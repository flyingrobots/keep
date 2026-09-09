//! This module owns admitted Golden File Worldline production crash fixtures.

use std::ops::Range;

use xtask::protocol_admission::{EmptyHex, decode_lower_hex};

use super::super::DurabilityCrashMatrixError;

const SEGMENT_HEX: &str =
    include_str!("../../../../conformance/segment-store/v1/one-zero-segment.hex");
const CATALOG_HEX: &str =
    include_str!("../../../../conformance/segment-store/v1/one-zero-catalog.hex");
const HEAD_HEX: &str = include_str!("../../../../conformance/segment-store/v1/one-zero-head.hex");
const BUNDLE_SEGMENT_HEX: &str =
    include_str!("../../../../conformance/segment-store/v1/one-zero-bundle-segment.hex");
const BUNDLE_CATALOG_HEX: &str =
    include_str!("../../../../conformance/segment-store/v1/one-zero-bundle-catalog.hex");
const BUNDLE_HEAD_HEX: &str =
    include_str!("../../../../conformance/segment-store/v1/one-zero-bundle-head.hex");
const RETENTION_ROOT_HEX: &str =
    include_str!("../../../../conformance/segment-store/v2/one-anchor-root.hex");
/// Pool name of the bundle segment the retention root's closure references.
pub(in crate::durability_crash_matrix) const BUNDLE_SEGMENT_NAME: &str =
    "221f6745cd8a5221c9a87c3707593608479282b54a4a74d0e753fd76f70e8db2.seg";
/// Pool name of the generation-one bundle catalog.
pub(in crate::durability_crash_matrix) const BUNDLE_CATALOG_NAME: &str =
    "0000000000000001-0b7cad1b6de663d34beacbc214db7497f2e36ab6b08dfbd5febbc8d06a418811.cat";

pub(in crate::durability_crash_matrix) const SEGMENT_POOL_PATH: &str =
    "segments/b7542dced2ab770894a14d1d04b066e3a899942602c5986d35ba6df6c1a35cfc.seg";
pub(in crate::durability_crash_matrix) const CATALOG_POOL_PATH: &str = "catalogs/0000000000000001-04b82519b0399baefd0b9c0f32a871052e4c47e3a00226ab03b21661470f7320.cat";

pub(in crate::durability_crash_matrix) struct GoldenFixture {
    bytes: Vec<u8>,
}

impl GoldenFixture {
    pub(in crate::durability_crash_matrix) fn segment() -> Result<Self, DurabilityCrashMatrixError>
    {
        Self::decode("segment", SEGMENT_HEX, 337)
    }

    pub(in crate::durability_crash_matrix) fn catalog() -> Result<Self, DurabilityCrashMatrixError>
    {
        Self::decode("catalog", CATALOG_HEX, 352)
    }

    pub(in crate::durability_crash_matrix) fn bundle_segment()
    -> Result<Self, DurabilityCrashMatrixError> {
        Self::decode("bundle segment", BUNDLE_SEGMENT_HEX, 701)
    }

    pub(in crate::durability_crash_matrix) fn bundle_catalog()
    -> Result<Self, DurabilityCrashMatrixError> {
        Self::decode("bundle catalog", BUNDLE_CATALOG_HEX, 512)
    }

    pub(in crate::durability_crash_matrix) fn bundle_head()
    -> Result<Self, DurabilityCrashMatrixError> {
        Self::decode("bundle head", BUNDLE_HEAD_HEX, 128)
    }

    pub(in crate::durability_crash_matrix) fn retention_root()
    -> Result<Self, DurabilityCrashMatrixError> {
        Self::decode("retention root", RETENTION_ROOT_HEX, 378)
    }

    pub(in crate::durability_crash_matrix) fn head() -> Result<Self, DurabilityCrashMatrixError> {
        Self::decode("head", HEAD_HEX, 128)
    }

    pub(in crate::durability_crash_matrix) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(in crate::durability_crash_matrix) fn range(
        &self,
        range: Range<usize>,
    ) -> Result<&[u8], DurabilityCrashMatrixError> {
        self.bytes
            .get(range)
            .ok_or(DurabilityCrashMatrixError::FixtureRange)
    }

    pub(in crate::durability_crash_matrix) fn prefix(
        &self,
        end: usize,
    ) -> Result<&[u8], DurabilityCrashMatrixError> {
        self.range(0..end)
    }

    fn decode(
        artifact: &'static str,
        encoded: &str,
        length: usize,
    ) -> Result<Self, DurabilityCrashMatrixError> {
        let hex = encoded
            .strip_suffix('\n')
            .ok_or(DurabilityCrashMatrixError::FixtureTerminator { artifact })?;
        let bytes = decode_lower_hex(hex, length, EmptyHex::Refuse)
            .map_err(|source| DurabilityCrashMatrixError::Fixture { artifact, source })?;
        if bytes.len() != length {
            return Err(DurabilityCrashMatrixError::FixtureLength {
                artifact,
                expected: length,
                observed: bytes.len(),
            });
        }
        Ok(Self { bytes })
    }
}
