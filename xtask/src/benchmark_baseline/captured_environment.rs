//! This module owns captured source, compiler and host measurement coordinates.

use std::num::NonZeroUsize;

#[derive(Eq, PartialEq)]
pub(super) struct CapturedEnvironment {
    pub(super) commit: String,
    pub(super) tree: &'static str,
    pub(super) rustc_version: String,
    pub(super) target_triple: String,
    pub(super) host: CapturedHost,
}

#[derive(Eq, PartialEq)]
pub(super) struct CapturedHost {
    pub(super) os_description: String,
    pub(super) cpu_model: String,
    pub(super) logical_cpu_count: NonZeroUsize,
}
