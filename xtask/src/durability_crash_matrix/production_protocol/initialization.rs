//! This module owns the crash children' initialized writer and read policies.

use std::path::Path;

use keep::{
    CatalogRestartByteLimit, CatalogRestartPolicy, FilesystemVersionTwoAdmission,
    FilesystemVersionTwoCatalogPublisher, LayoutEntryLimit, SegmentReadPolicy, SegmentRecordLimit,
};

use super::{DurabilityCrashMatrixError, verification};

const RESTART_BYTE_LIMIT: u64 = 1_048_576;

/// Initializes the single-format store and opens its writer, as production does.
pub(super) fn publisher(
    store_root: &Path,
) -> Result<FilesystemVersionTwoCatalogPublisher, DurabilityCrashMatrixError> {
    let admission = FilesystemVersionTwoAdmission::initialize(store_root)
        .map_err(|source| verification("initialize crash catalog store", source))?
        .into_admission();
    FilesystemVersionTwoCatalogPublisher::open(admission, restart_policy()?)
        .map_err(|source| DurabilityCrashMatrixError::io("open crash catalog publisher", source))
}

pub(super) fn restart_policy() -> Result<CatalogRestartPolicy, DurabilityCrashMatrixError> {
    let byte_limit = CatalogRestartByteLimit::new(RESTART_BYTE_LIMIT)
        .map_err(|source| verification("construct crash restart byte limit", source))?;
    Ok(CatalogRestartPolicy::new(segment_policy(), byte_limit))
}

pub(in crate::durability_crash_matrix) const fn segment_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}
