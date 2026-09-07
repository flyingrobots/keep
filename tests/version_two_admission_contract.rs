//! Version-two writer authority is a distinct type that version-one publishers cannot consume.

const RETENTION_AUTHORITY: &str =
    include_str!("../src/adapters/retention/filesystem_retention_authority.rs");
const CATALOG_PUBLISHER: &str = include_str!("../src/adapters/filesystem_catalog_publisher.rs");
const STORE_INITIALIZER: &str = include_str!("../src/adapters/filesystem_store_initializer.rs");
const VERSION_TWO_ADMISSION: &str =
    include_str!("../src/adapters/filesystem_version_two_admission.rs");
const MIGRATION_AUTHORITY: &str =
    include_str!("../src/adapters/store_migration/filesystem_migration_authority.rs");

#[test]
fn retention_publication_consumes_only_version_two_authority() {
    assert!(RETENTION_AUTHORITY.contains("pub fn open(admission: FilesystemVersionTwoAdmission)"));
    assert!(!RETENTION_AUTHORITY.contains("admission: FilesystemPlatformAdmission"));
}

#[test]
fn version_one_publishers_consume_only_version_one_authority() {
    assert!(CATALOG_PUBLISHER.contains("admission: FilesystemPlatformAdmission,"));
    assert!(!CATALOG_PUBLISHER.contains("FilesystemVersionTwoAdmission"));
    assert!(MIGRATION_AUTHORITY.contains("admission: FilesystemPlatformAdmission,"));
    assert!(!MIGRATION_AUTHORITY.contains("FilesystemVersionTwoAdmission"));
}

#[test]
fn version_two_reopen_produces_only_version_two_authority() {
    assert!(!STORE_INITIALIZER.contains("fn reopen_version_two"));
    assert!(VERSION_TWO_ADMISSION.contains("pub struct FilesystemVersionTwoAdmission"));
    assert!(VERSION_TWO_ADMISSION.contains("pub fn reopen(store_root: &Path)"));
    assert!(VERSION_TWO_ADMISSION.contains("filesystem_version_two_records::admit"));
    assert!(VERSION_TWO_ADMISSION.contains("filesystem_platform_profile::open_version_two"));
}
