//! This module owns bounded, deterministic Git path inventory.

mod error;
mod path_stream;
mod process;

pub(crate) use error::{GitInventoryError, GitOutputUnit};
pub(crate) use path_stream::GitPath;
pub(crate) use process::paths_with;
