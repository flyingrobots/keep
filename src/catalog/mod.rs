//! Catalog-generation domain coordinates and transition laws.
//!
//! This module owns semantic catalog generations. It does not own catalog byte
//! encoding, physical paths, filesystem publication, recovery, or retention.

mod generation;
mod generation_error;

pub use generation::CatalogGeneration;
pub use generation_error::CatalogGenerationError;
