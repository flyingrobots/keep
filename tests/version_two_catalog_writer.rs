//! Version-two catalog writer and stage-recovery laws on initialized stores.
//!
//! Every law initializes a real store, publishes its first catalog and a
//! retention root through the public writer, and then drives the version-two writer and recovery tools
//! through their public production entry points.

#![cfg(target_os = "linux")]

#[path = "version_two_catalog_writer/suite.rs"]
mod suite;
