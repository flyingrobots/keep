//! This module owns canonical catalog verification over caller-supplied bytes.
#![expect(
    clippy::result_large_err,
    reason = "preserve complete bounded refusal coordinates"
)]

use super::verification_ingress::catalog_error;
use crate::{
    AdmittedSegment, CatalogRestartError, ChecksummedCatalog, ChecksummedPublicationHead,
    VerificationDepth, VerificationError, VerificationReport,
};

/// Admits the supplied publication head, catalog and immutable segments.
///
/// Reports only the catalog evidence supported by `CatalogSnapshot::verify`.
/// Prerequisite admission validates all selected physical bindings, even for a
/// framing request. It scans catalog and referenced records, allocates bounded
/// catalog-entry/segment indexes, and performs no filesystem I/O or mutation.
/// Supplied segment owners and bytes must outlive this call, not the report.
///
/// # Errors
///
/// Preserves original head, catalog, physical binding and resource failures in
/// distinct missing/corrupt/operational outcomes; unsupported depth refuses.
pub fn verify_catalog_bytes(
    head: &[u8],
    catalog: &[u8],
    segments: &[AdmittedSegment<'_>],
    requested: VerificationDepth,
) -> Result<VerificationReport, VerificationError> {
    let head = ChecksummedPublicationHead::decode(head)
        .map_err(|source| catalog_error(CatalogRestartError::Head { source }))?;
    let catalog = ChecksummedCatalog::decode(catalog)
        .map_err(|source| catalog_error(CatalogRestartError::Catalog { source }))?;
    let admitted = catalog.admit(segments).map_err(|source| {
        catalog_error(CatalogRestartError::CatalogAdmission {
            source: Box::new(source),
        })
    })?;
    let view = head
        .admit(admitted)
        .map_err(|source| catalog_error(CatalogRestartError::Snapshot { source }))?;
    view.verify(requested).map_err(Into::into)
}
