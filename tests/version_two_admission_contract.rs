//! Writer authority over a store comes only from single-format admission.

const RETENTION_AUTHORITY: &str =
    include_str!("../src/adapters/retention/filesystem_retention_authority.rs");
const CATALOG_PUBLISHER: &str =
    include_str!("../src/adapters/filesystem_version_two_catalog_publisher.rs");
const VERSION_TWO_ADMISSION: &str =
    include_str!("../src/adapters/filesystem_version_two_admission.rs");
const ADMISSION_ERROR: &str =
    include_str!("../src/adapters/filesystem_platform_admission_error.rs");

/// The type-level proofs are the `compile_fail` doctests on
/// `FilesystemRetentionPublicationAuthority::open` and
/// `FilesystemVersionTwoCatalogPublisher::open`, which refuse a bare
/// `FilesystemWriterLock`. These markers only keep each writer on the
/// admission side of the boundary without pinning a signature.
#[test]
fn every_writer_consumes_admitted_authority() {
    assert!(RETENTION_AUTHORITY.contains("FilesystemVersionTwoAdmission"));
    assert!(CATALOG_PUBLISHER.contains("admission: FilesystemVersionTwoAdmission"));
}

#[test]
fn reopen_admits_the_marker_under_the_production_profile() {
    assert!(VERSION_TWO_ADMISSION.contains("pub struct FilesystemVersionTwoAdmission"));
    assert!(VERSION_TWO_ADMISSION.contains("pub fn reopen(store_root: &Path)"));
    assert!(VERSION_TWO_ADMISSION.contains("pub fn initialize("));
    assert!(VERSION_TWO_ADMISSION.contains("filesystem_format_marker::admit"));
    assert!(VERSION_TWO_ADMISSION.contains("filesystem_platform_profile::open_version_two"));
}

/// Admission refusals grow with the platform surface, so the public error is
/// non-exhaustive and a new refusal is not a breaking change.
#[test]
fn admission_error_is_non_exhaustive() {
    assert!(
        ADMISSION_ERROR.contains("#[non_exhaustive]\npub enum FilesystemPlatformAdmissionError")
    );
}
