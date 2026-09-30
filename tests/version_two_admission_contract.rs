//! Version-two writer authority is a distinct type that version-one publishers
//! cannot consume. The one deliberate exception is the catalog publisher's
//! `open_version_two`, which compaction uses to publish catalog successors
//! over a migrated root: it consumes a version-two admission by value and
//! never accepts version-one authority for a version-two root.

const RETENTION_AUTHORITY: &str =
    include_str!("../src/adapters/retention/filesystem_retention_authority.rs");
const CATALOG_PUBLISHER: &str = include_str!("../src/adapters/filesystem_catalog_publisher.rs");
const STORE_INITIALIZER: &str = include_str!("../src/adapters/filesystem_store_initializer.rs");
const VERSION_TWO_ADMISSION: &str =
    include_str!("../src/adapters/filesystem_version_two_admission.rs");
const MIGRATION_AUTHORITY: &str =
    include_str!("../src/adapters/store_migration/filesystem_migration_authority.rs");
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
    assert!(CATALOG_PUBLISHER.contains("admission: FilesystemPlatformAdmission"));
    assert!(MIGRATION_AUTHORITY.contains("FilesystemPlatformAdmission"));
    assert!(!MIGRATION_AUTHORITY.contains("FilesystemVersionTwoAdmission"));
}

#[test]
fn catalog_successors_consume_version_two_authority_only_as_a_proof() {
    assert!(
        CATALOG_PUBLISHER.contains(
            "pub fn open_version_two(\n        admission: FilesystemVersionTwoAdmission,"
        )
    );
    assert_eq!(
        CATALOG_PUBLISHER
            .matches("FilesystemVersionTwoAdmission")
            .count(),
        2,
        "the import and the one by-value parameter"
    );
    assert!(!CATALOG_PUBLISHER.contains("FilesystemVersionTwoAdmission::"));
}

#[test]
fn version_two_reopen_produces_only_version_two_authority() {
    assert!(!STORE_INITIALIZER.contains("fn reopen_version_two"));
    assert!(VERSION_TWO_ADMISSION.contains("pub struct FilesystemVersionTwoAdmission"));
    assert!(VERSION_TWO_ADMISSION.contains("pub fn reopen(store_root: &Path)"));
    assert!(VERSION_TWO_ADMISSION.contains("filesystem_version_two_records::admit"));
    assert!(VERSION_TWO_ADMISSION.contains("filesystem_platform_profile::open_version_two"));
}

/// Admission refusals grow with the platform surface (`MigrationRecord` and
/// `RootIdentityChanged` arrived after the first release candidate), so the
/// public error is non-exhaustive and a new refusal is not a breaking change.
#[test]
fn admission_error_is_non_exhaustive() {
    assert!(
        ADMISSION_ERROR.contains("#[non_exhaustive]\npub enum FilesystemPlatformAdmissionError")
    );
}
