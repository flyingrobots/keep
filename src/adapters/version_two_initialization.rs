//! This module owns direct initialization of a version-two store under writer authority.
//!
//! Initialization is the only way a store comes into existence. It is a
//! recoverable state machine: every interrupted state is a classified prefix
//! that a later call resumes, every other nonempty root is refused before any
//! mutation, and the store admits only after one named durable commit point.
//! The protocol, its states, and its crash matrix are specified in
//! `docs/formats/segment-store-v2/initialization.md`.

mod initialization_entries;
mod initialization_error;
mod initialization_execution;
mod initialization_namespace;
mod initialization_observation;
mod initialization_phase;
mod initialization_prefix;
#[cfg(test)]
mod initialization_prefix_tests;
mod initialization_reader_fence;
mod initialization_receipt;
mod initialization_refusal;
#[cfg(test)]
mod initialization_resume_tests;
#[cfg(test)]
mod initialization_test_fixture;

pub use initialization_error::VersionTwoInitializationError;
pub use initialization_phase::{VersionTwoInitializationDirectory, VersionTwoInitializationPhase};
pub use initialization_prefix::VersionTwoInitializationPrefix;
pub use initialization_receipt::VersionTwoInitialization;
pub use initialization_refusal::VersionTwoInitializationRefusal;

pub(super) use initialization_execution::{PhaseObserver, initialize};

/// Where an initialization observer runs relative to one phase.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VersionTwoInitializationBoundary {
    /// Before the phase begins.
    Before,
    /// After the phase returned successfully.
    After,
}

/// Repository-task observer of initialization phases.
///
/// Returning an error fails the phase at that boundary exactly as a storage
/// failure would; the crash matrix blocks inside it for process death.
#[cfg(any(test, feature = "repository-tasks"))]
#[doc(hidden)]
pub type VersionTwoInitializationObserver = Box<
    dyn FnMut(
        VersionTwoInitializationPhase,
        VersionTwoInitializationBoundary,
    ) -> std::io::Result<()>,
>;
