//! Version-two writer authority is a distinct type that version-one publishers cannot consume.

const RETENTION_AUTHORITY: &str =
    include_str!("../src/adapters/retention/filesystem_retention_authority.rs");
const CATALOG_PUBLISHER: &str = include_str!("../src/adapters/filesystem_catalog_publisher.rs");
const STORE_INITIALIZER: &str = include_str!("../src/adapters/filesystem_store_initializer.rs");
const VERSION_TWO_ADMISSION: &str =
    include_str!("../src/adapters/filesystem_version_two_admission.rs");
const ADMISSION_ERROR: &str =
    include_str!("../src/adapters/filesystem_platform_admission_error.rs");

/// The type-level proof is the `compile_fail` doctest on
/// `FilesystemRetentionPublicationAuthority::open`, which refuses a
/// `FilesystemPlatformAdmission` argument at compile time. These markers only
/// keep each file on its side of the boundary without pinning a signature.
#[test]
fn retention_publication_consumes_only_version_two_authority() {
    assert!(RETENTION_AUTHORITY.contains("FilesystemVersionTwoAdmission"));
    assert!(!RETENTION_AUTHORITY.contains("admission: FilesystemPlatformAdmission"));
}

#[test]
fn version_one_publishers_consume_only_version_one_authority() {
    assert!(CATALOG_PUBLISHER.contains("FilesystemPlatformAdmission"));
    assert!(!CATALOG_PUBLISHER.contains("FilesystemVersionTwoAdmission"));
}

#[test]
fn version_two_reopen_produces_only_version_two_authority() {
    assert!(!STORE_INITIALIZER.contains("fn reopen_version_two"));
    assert!(!STORE_INITIALIZER.contains("FilesystemVersionTwoAdmission"));
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
