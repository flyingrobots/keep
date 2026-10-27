//! Catalog writer and stage-recovery laws on initialized stores.
//!
//! Every law initializes a real store, publishes its first catalog and a
//! retention root through the public writer, and then drives the writer and recovery tools
//! through their public production entry points.

#![cfg(target_os = "linux")]

#[path = "catalog_writer/suite.rs"]
mod suite;
