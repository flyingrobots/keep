//! Semantic garbage-collection coordinates.
//!
//! This module owns the checked garbage-collection generation. It does not
//! own record encoding, retirement planning, execution, or recovery.

mod generation;
mod generation_error;

pub use generation::GcGeneration;
pub use generation_error::GcGenerationError;
