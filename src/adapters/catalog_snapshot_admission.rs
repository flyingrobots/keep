//! Exact publication-head to admitted-catalog snapshot binding.

use super::{AdmittedCatalog, CatalogSnapshot, CatalogSnapshotError, ChecksummedPublicationHead};

pub(super) fn admit<'head, 'catalog, 'records>(
    head: ChecksummedPublicationHead<'head>,
    catalog: AdmittedCatalog<'catalog, 'records>,
) -> Result<CatalogSnapshot<'head, 'catalog, 'records>, CatalogSnapshotError> {
    let expected = head.generation();
    let observed = catalog.generation();
    if observed != expected {
        return Err(CatalogSnapshotError::Generation { expected, observed });
    }
    let expected = head.catalog_length();
    let observed = catalog.length();
    if observed != expected {
        return Err(CatalogSnapshotError::CatalogLength { expected, observed });
    }
    let expected = head.catalog_digest();
    let observed = catalog.digest();
    if observed != expected {
        return Err(CatalogSnapshotError::CatalogDigest { expected, observed });
    }
    Ok(CatalogSnapshot::new(head, catalog))
}
