//! Isolated child-process setup for descriptor-bound working directories.
//!
//! This crate contains Keep's only admitted unsafe boundary. It exists because
//! [`std::process::Command`] exposes child setup through an unsafe hook, while
//! an opened directory descriptor is required to prevent pathname-replacement
//! races. No filesystem, storage, identity, or durable-format code belongs
//! here.

#![allow(
    unsafe_code,
    reason = "this dedicated crate owns the reviewed pre-exec fchdir boundary"
)]

use std::os::fd::OwnedFd;
use std::os::unix::process::CommandExt;
use std::process::Command;

/// Configures `command` to enter `directory` after fork and before exec.
///
/// The descriptor remains owned by the command hook. Any `fchdir` failure is
/// returned by the later spawn operation. The hook performs no allocation,
/// locking, I/O buffering, or user callback.
pub fn set_working_directory(command: &mut Command, directory: OwnedFd) {
    // SAFETY: POSIX specifies fchdir as async-signal-safe. The hook captures an
    // owned descriptor and performs exactly that operation between fork and
    // exec. It does not allocate, lock, inspect ambient paths, or call user
    // code. Rustix supplies the checked safe wrapper around the system call.
    unsafe {
        command.pre_exec(move || {
            rustix::process::fchdir(&directory)?;
            Ok(())
        });
    }
}
