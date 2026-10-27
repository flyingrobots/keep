//! This module owns proof that a filesystem root passed store admission.

use std::path::Path;

use cap_fs_ext::DirExt;
#[cfg(any(test, feature = "repository-tasks"))]
use cap_std::ambient_authority;
use cap_std::fs::Dir;

#[cfg(any(test, feature = "repository-tasks"))]
use super::StoreInitializationObserver;
use super::store_initialization::{self as initialization, PhaseObserver};
use super::{
    FilesystemStoreAdmissionError, FilesystemWriterLock, StoreInitialization,
    StoreInitializationError, filesystem_format_marker, filesystem_platform_profile,
    filesystem_store_namespace,
};

/// Exclusive writer authority over a committed store root.
///
/// A value proves the production platform, the exact committed namespace, and
/// the canonical `FORMAT` marker under the writer lock; every writer consumes
/// one, so no writer runs against a root without the marker. Fields are
/// private so only admission and initialization can create values. The admitted `retention`, `roots`, and
/// `manifests` capabilities are retained, so the authority built from this
/// value operates on the directories that passed admission and not on whatever
/// those names resolve to later.
///
/// Admission binds the store to no directory: nothing persisted records the
/// root's device, inode, mount, or path, so a quiescent copy opens wherever
/// it is placed. Live authority is the writer lock and the pinned
/// capabilities of this process.
#[must_use]
pub struct FilesystemStoreAdmission {
    lock: FilesystemWriterLock,
    retention: Dir,
    roots: Dir,
    manifests: Dir,
}

impl FilesystemStoreAdmission {
    /// Initializes a store at `store_root`, or resumes an interrupted initialization.
    ///
    /// The root must exist. An empty root, or a recoverable prefix that an
    /// interrupted call left, becomes a committed store with no catalog
    /// `HEAD`; every other root is refused before it is mutated. The call
    /// admits the production platform, holds exclusive writer authority from
    /// its first mutation, and returns only after the commit point, the
    /// removal of `FORMAT.next`, the final root synchronization, and complete
    /// admission of the committed root. It is synchronous, allocates no
    /// content-sized memory, and may block on filesystem I/O.
    ///
    /// # Errors
    ///
    /// Returns [`StoreInitializationError`] with the exact platform,
    /// writer-lock, refusal, phase, or admission failure. A committed store
    /// refuses as
    /// [`AlreadyInitialized`](super::StoreInitializationRefusal::AlreadyInitialized);
    /// a concurrent attempt refuses with a busy writer lock.
    pub fn initialize(store_root: &Path) -> Result<StoreInitialization, StoreInitializationError> {
        let root = filesystem_platform_profile::open(store_root)
            .map_err(|source| StoreInitializationError::Platform { source })?;
        initialize_root(&root, &mut |_, _| Ok(()))
    }

    /// Initializes without platform admission for repository tasks.
    ///
    /// Every protocol law after platform admission still applies. Production
    /// callers use [`Self::initialize`].
    ///
    /// # Errors
    ///
    /// Returns [`StoreInitializationError`] exactly as [`Self::initialize`]
    /// does after platform admission.
    #[cfg(any(test, feature = "repository-tasks"))]
    pub fn initialize_unchecked_for_repository_tasks(
        store_root: &Path,
    ) -> Result<StoreInitialization, StoreInitializationError> {
        initialize_root(&open_unchecked(store_root)?, &mut |_, _| Ok(()))
    }

    /// Initializes without platform admission, notifying `observer` around each phase.
    ///
    /// The crash matrix blocks inside the observer for process death; an
    /// observer error fails the phase at that boundary.
    ///
    /// # Errors
    ///
    /// Returns [`StoreInitializationError`] as
    /// [`Self::initialize_unchecked_for_repository_tasks`] does, or the
    /// observer's error at its phase.
    #[cfg(any(test, feature = "repository-tasks"))]
    #[doc(hidden)]
    pub fn initialize_observed_for_repository_tasks(
        store_root: &Path,
        mut observer: StoreInitializationObserver,
    ) -> Result<StoreInitialization, StoreInitializationError> {
        initialize_root(&open_unchecked(store_root)?, &mut *observer)
    }

    /// Reacquires writer authority over one committed store.
    ///
    /// The call mutates no protocol state. It admits the production platform
    /// for every protocol directory, acquires the existing writer
    /// lock, requires the exact committed root namespace, and admits the
    /// canonical `FORMAT` marker. A store with no catalog `HEAD` admits.
    /// Retention adapters perform content-level validation under the
    /// returned authority. The synchronous call may block on filesystem I/O.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemStoreAdmissionError`] with the exact platform,
    /// writer-lock, namespace, or format-marker boundary and preserved source.
    /// A root whose initialization has not committed refuses at the namespace
    /// boundary.
    pub fn reopen(store_root: &Path) -> Result<Self, FilesystemStoreAdmissionError> {
        let root = filesystem_platform_profile::open(store_root)
            .map_err(|source| FilesystemStoreAdmissionError::Platform { source })?;
        Self::admit(root)
    }

    /// Reopens a committed root without platform admission for repository tasks.
    ///
    /// The crash matrix and other repository tools run on hosts outside the
    /// admitted Linux profile; every namespace and record law still applies.
    /// Production callers use [`Self::reopen`].
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemStoreAdmissionError`] exactly as [`Self::reopen`]
    /// does for every boundary after platform admission.
    #[cfg(any(test, feature = "repository-tasks"))]
    pub fn reopen_unchecked_for_repository_tasks(
        store_root: &Path,
    ) -> Result<Self, FilesystemStoreAdmissionError> {
        let root = Dir::open_ambient_dir(store_root, ambient_authority())
            .map_err(|source| FilesystemStoreAdmissionError::Platform { source })?;
        Self::admit(root)
    }

    /// Releases the writer lock and the three pinned retention capabilities.
    pub(super) fn into_parts(self) -> (FilesystemWriterLock, Dir, Dir, Dir) {
        (self.lock, self.retention, self.roots, self.manifests)
    }

    fn admit(root: Dir) -> Result<Self, FilesystemStoreAdmissionError> {
        admit_locked(root, filesystem_store_namespace::admit_store).map(Self::from_parts)
    }

    fn from_parts((lock, retention, roots, manifests): AdmittedStoreParts) -> Self {
        Self {
            lock,
            retention,
            roots,
            manifests,
        }
    }
}

/// Writer authority and the three pinned retention capabilities of one admitted root.
pub(super) type AdmittedStoreParts = (FilesystemWriterLock, Dir, Dir, Dir);

#[cfg(any(test, feature = "repository-tasks"))]
fn open_unchecked(store_root: &Path) -> Result<Dir, StoreInitializationError> {
    Dir::open_ambient_dir(store_root, ambient_authority())
        .map_err(|source| StoreInitializationError::Platform { source })
}

/// Runs the initialization protocol, then admits the committed root under its lock.
fn initialize_root(
    root: &Dir,
    observer: PhaseObserver<'_>,
) -> Result<StoreInitialization, StoreInitializationError> {
    let (lock, found) = initialization::initialize(root, observer)?;
    let parts = admit_under_lock(lock, filesystem_store_namespace::admit_store)
        .map_err(|source| StoreInitializationError::Admission { source })?;
    Ok(StoreInitialization::new(
        FilesystemStoreAdmission::from_parts(parts),
        found,
    ))
}

/// Acquires writer authority, then admits the root under `namespace`.
///
/// Forward writers pass the exact store namespace law; catalog-stage
/// recovery passes the law that also admits a retained `head.next`. Every
/// other check is identical: the `FORMAT` marker and the pinned retention
/// directories.
pub(super) fn admit_locked(
    root: Dir,
    namespace: fn(&Dir) -> std::io::Result<()>,
) -> Result<AdmittedStoreParts, FilesystemStoreAdmissionError> {
    let lock = FilesystemWriterLock::try_acquire_in(root)
        .map_err(|source| FilesystemStoreAdmissionError::WriterLock { source })?;
    admit_under_lock(lock, namespace)
}

fn admit_under_lock(
    lock: FilesystemWriterLock,
    namespace: fn(&Dir) -> std::io::Result<()>,
) -> Result<AdmittedStoreParts, FilesystemStoreAdmissionError> {
    let directory = lock
        .clone_directory()
        .map_err(|source| FilesystemStoreAdmissionError::Namespace { source })?;
    admit_root(&directory, namespace)?;
    let retention = pin(&directory, "retention")?;
    let roots = pin(&retention, "roots")?;
    let manifests = pin(&retention, "manifests")?;
    Ok((lock, retention, roots, manifests))
}

/// Admits the namespace and the `FORMAT` marker without mutation.
pub(super) fn admit_root(
    directory: &Dir,
    namespace: fn(&Dir) -> std::io::Result<()>,
) -> Result<(), FilesystemStoreAdmissionError> {
    namespace(directory).map_err(|source| FilesystemStoreAdmissionError::Namespace { source })?;
    filesystem_format_marker::admit(directory)
        .map_err(|source| FilesystemStoreAdmissionError::FormatMarker { source })
}

/// Pins one admitted protocol directory without following links.
fn pin(parent: &Dir, name: &str) -> Result<Dir, FilesystemStoreAdmissionError> {
    parent
        .open_dir_nofollow(name)
        .map_err(|source| FilesystemStoreAdmissionError::Namespace { source })
}
