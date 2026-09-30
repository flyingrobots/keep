//! This module owns the blocking storage capability port for retention recovery.

use std::io;

/// Durable capabilities retention recovery executes, one per plan step.
///
/// Each capability owns its complete effect and the synchronization that makes
/// it durable, so an implementation cannot report a step as done before its
/// evidence would survive process death. Every capability is called at most
/// once per plan, in plan order, and never after a refused capability.
pub trait RetentionRecoveryStorage {
    /// Removes a truncated `head.next` and synchronizes `retention`.
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure; the stage must remain when it fails.
    fn discard_head_stage(&mut self) -> io::Result<()>;

    /// Removes a truncated, never linked `manifest.next` and synchronizes `retention`.
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure; the stage must remain when it fails.
    fn discard_manifest_stage(&mut self) -> io::Result<()>;

    /// Removes a truncated, never linked `root.next` and synchronizes `retention`.
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure; the stage must remain when it fails.
    fn discard_root_stage(&mut self) -> io::Result<()>;

    /// Admits the staged root's namespace directory, links the complete root
    /// stage into it without replacement, and synchronizes both directories.
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure or a refusal of a conflicting entry.
    fn link_root(&mut self) -> io::Result<()>;

    /// Links the complete manifest stage into the manifest pool without
    /// replacement and synchronizes the pool.
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure or a refusal of a conflicting entry.
    fn link_manifest(&mut self) -> io::Result<()>;

    /// Replaces `retention/HEAD` with the complete head stage atomically and
    /// synchronizes `retention`.
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure.
    fn finalize_head(&mut self) -> io::Result<()>;

    /// Removes the retained root stage after proving its pool link and
    /// synchronizes `retention`.
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure or a refusal when the link is not proven.
    fn remove_root_stage(&mut self) -> io::Result<()>;

    /// Removes the retained manifest stage after proving its pool link and
    /// synchronizes `retention`.
    ///
    /// # Errors
    ///
    /// Returns the exact filesystem failure or a refusal when the link is not proven.
    fn remove_manifest_stage(&mut self) -> io::Result<()>;
}
