//! Catalog-generation domain coordinates and transition laws.
//!
//! This module owns semantic catalog generations and checked physical catalog
//! coordinates. It does not own catalog byte encoding, physical paths,
//! filesystem publication, recovery, or retention.

mod digest;
mod generation;
mod generation_error;
mod length;
mod length_error;

pub use digest::CatalogDigest;
pub use generation::CatalogGeneration;
pub use generation_error::CatalogGenerationError;
pub use length::CatalogLength;
pub use length_error::CatalogLengthError;
