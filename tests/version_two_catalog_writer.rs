//! Version-two catalog writer and stage-recovery laws on migrated stores.
//!
//! Every law builds a real version-one store, migrates it, publishes a
//! retention root, and then drives the version-two writer and recovery tools
//! through their public production entry points.

#![cfg(target_os = "linux")]

#[path = "version_two_catalog_writer/suite.rs"]
mod suite;
