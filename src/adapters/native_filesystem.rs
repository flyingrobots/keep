//! This module owns compile-time selection of the native filesystem adapter.

#[cfg(target_os = "linux")]
pub(super) use super::linux_filesystem::LinuxFilesystem as NativeFilesystem;

#[cfg(not(target_os = "linux"))]
pub(super) use super::unsupported_filesystem::UnsupportedFilesystem as NativeFilesystem;
