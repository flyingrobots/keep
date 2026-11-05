//! Writer authority over a store comes only from single-format admission.

const RETENTION_AUTHORITY: &str =
    include_str!("../src/adapters/retention/filesystem_retention_authority.rs");
const CATALOG_PUBLISHER: &str = include_str!("../src/adapters/filesystem_catalog_publisher.rs");
const STORE_ADMISSION: &str = include_str!("../src/adapters/filesystem_store_admission.rs");
const ADMISSION_ERROR: &str = include_str!("../src/adapters/filesystem_store_admission_error.rs");

/// The type-level proofs are the `compile_fail` doctests on
/// `FilesystemRetentionPublicationAuthority::open` and
/// `FilesystemCatalogPublisher::open`, which refuse a bare
/// `FilesystemWriterLock`. These markers only keep each writer on the
/// admission side of the boundary without pinning a signature.
#[test]
fn every_writer_consumes_admitted_authority() {
    assert!(RETENTION_AUTHORITY.contains("FilesystemStoreAdmission"));
    assert!(CATALOG_PUBLISHER.contains("admission: FilesystemStoreAdmission"));
}

#[test]
fn reopen_admits_the_marker_under_the_production_profile() {
    assert!(STORE_ADMISSION.contains("pub struct FilesystemStoreAdmission"));
    assert!(STORE_ADMISSION.contains("pub fn reopen(store_root: &Path)"));
    assert!(STORE_ADMISSION.contains("pub fn initialize("));
    assert!(STORE_ADMISSION.contains("filesystem_format_marker::admit"));
    assert!(STORE_ADMISSION.contains("filesystem_platform_profile::open"));
}

/// Admission refusals grow with the platform surface, so the public error is
/// non-exhaustive and a new refusal is not a breaking change.
#[test]
fn admission_error_is_non_exhaustive() {
    assert!(ADMISSION_ERROR.contains("#[non_exhaustive]\npub enum FilesystemStoreAdmissionError"));
}
