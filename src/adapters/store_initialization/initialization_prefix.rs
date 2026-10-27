//! This module owns the lawful states a root can hold before initialization commits.

use super::StoreInitializationDirectory;

/// One recoverable initialization state, observed under writer authority.
///
/// Each variant is exactly one forward state of the protocol in
/// `docs/formats/store/initialization.md`. A root in any other
/// state is refused, and a committed root is refused as already initialized,
/// so no variant describes a committed store.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreInitializationPrefix {
    /// The root was empty.
    Empty,
    /// The root held only a zero-length `writer.lock`.
    Locked,
    /// `FORMAT.next` held a strict prefix of the canonical marker.
    IncompleteMarkerStage {
        /// Observed stage length, always below the marker length.
        length: usize,
    },
    /// `FORMAT.next` held the canonical marker; nothing else was created.
    MarkerStaged,
    /// The persistent reader fence existed; no directory was created.
    ReaderFenced,
    /// Every directory up to and including `last` existed, in order.
    Directories {
        /// The last directory present in creation order.
        last: StoreInitializationDirectory,
    },
    /// `FORMAT` named the same inode as the canonical `FORMAT.next`.
    MarkerLinked,
}
