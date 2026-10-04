//! This module owns the blocking storage capability port for retention recovery.

use super::RetentionStorageError;

/// Durable capabilities retention recovery executes, one per plan step.
///
/// Each capability owns its complete effect and the synchronization that makes
/// it durable, so an implementation cannot report a step as done before its
/// evidence would survive process death. Every capability is called at most
/// once per plan, in plan order, and never after a failed capability.
///
/// Failure is not rollback: report the failing boundary and known/uncertain
/// namespace effects through `RetentionStorageError::Operation` when available.
/// Missing progress is unreported effects, never proof of no mutation. A caller
/// must obtain fresh observation before another attempt.
pub trait RetentionRecoveryStorage {
    /// Reserved incomplete-head disposition capability; automatic disposal is deferred.
    ///
    /// # Errors
    ///
    /// Must refuse without mutation. The filesystem adapter returns `IncompleteDispositionRequired`.
    fn discard_head_stage(&mut self) -> Result<(), RetentionStorageError>;

    /// Reserved incomplete-manifest disposition capability; automatic disposal is deferred.
    ///
    /// # Errors
    ///
    /// Must refuse without mutation. The filesystem adapter returns `IncompleteDispositionRequired`.
    fn discard_manifest_stage(&mut self) -> Result<(), RetentionStorageError>;

    /// Reserved incomplete-root disposition capability; automatic disposal is deferred.
    ///
    /// # Errors
    ///
    /// Must refuse without mutation. The filesystem adapter returns `IncompleteDispositionRequired`.
    fn discard_root_stage(&mut self) -> Result<(), RetentionStorageError>;

    /// Admits the staged root's namespace directory, links the complete root
    /// stage into it without replacement, and synchronizes both directories.
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure or a refusal of a conflicting entry.
    fn link_root(&mut self) -> Result<(), RetentionStorageError>;

    /// Links the complete manifest stage into the manifest pool without
    /// replacement and synchronizes the pool.
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure or a refusal of a conflicting entry.
    fn link_manifest(&mut self) -> Result<(), RetentionStorageError>;

    /// Replaces `retention/HEAD` with the complete head stage atomically and
    /// synchronizes `retention`.
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure.
    fn finalize_head(&mut self) -> Result<(), RetentionStorageError>;

    /// Removes the retained root stage after proving its pool link and
    /// synchronizes `retention`.
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure or a refusal when the link is not proven.
    fn remove_root_stage(&mut self) -> Result<(), RetentionStorageError>;

    /// Removes the retained manifest stage after proving its pool link and
    /// synchronizes `retention`.
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure or a refusal when the link is not proven.
    fn remove_manifest_stage(&mut self) -> Result<(), RetentionStorageError>;
}
