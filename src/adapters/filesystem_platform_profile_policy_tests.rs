//! Mount-identity policy laws shared by every platform's test build.

use super::{MountIdentityPolicy, admit_mount_identity};

#[test]
fn production_identity_requires_a_reported_mount_identity() {
    assert_eq!(
        admit_mount_identity(MountIdentityPolicy::Required, true, 7),
        Some(7)
    );
    assert_eq!(
        admit_mount_identity(MountIdentityPolicy::Required, false, 7),
        None
    );
}

#[test]
fn bypass_identity_records_an_unreported_mount_identity_as_zero() {
    assert_eq!(
        admit_mount_identity(MountIdentityPolicy::Lenient, true, 7),
        Some(7)
    );
    assert_eq!(
        admit_mount_identity(MountIdentityPolicy::Lenient, false, 7),
        Some(0)
    );
}
