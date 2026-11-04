//! This module owns compile-time selection of the native filesystem adapter.

#[cfg(target_os = "linux")]
pub(super) use super::linux_filesystem::LinuxFilesystem as NativeFilesystem;

#[cfg(target_os = "macos")]
pub(super) use super::darwin_filesystem::DarwinFilesystem as NativeFilesystem;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub(super) use super::unsupported_filesystem::UnsupportedFilesystem as NativeFilesystem;
