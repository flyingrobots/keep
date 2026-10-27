//! Public laws of the single store format: direct initialization, portable
//! stores, and the integrated acceptance sequence that replaced migration.
//!
//! Every law creates its store through `FilesystemVersionTwoAdmission::initialize`
//! and drives the version-two writer, recovery tools, and durable reader
//! through their public production entry points.

#![cfg(target_os = "linux")]

#[path = "single_format/suite.rs"]
mod suite;
