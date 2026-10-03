//! This module owns one fenced, double-collected reader view of a version-two store.

use std::io;
use std::path::Path;

use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

use super::filesystem_retention_current::{self, ObservedRetentionState};
use super::filesystem_retention_pool_name as pool_name;
use super::{
    AdmittedRetentionRoot, FilesystemRetentionSnapshotError as Error, ReaderAttemptLimit,
    ReaderFence, RetentionSelectedRootRefusal, RetentionViewCoordinates, RetentionViewSource,
    collect_retention_view, root_header_decoder,
};
use crate::adapters::filesystem_exact_record::{self as exact_record, ExactRecordError};
use crate::adapters::filesystem_platform_profile::root_identity;
use crate::adapters::filesystem_version_two_admission::require_root_identity;
use crate::adapters::{
    CatalogRestartError, CatalogRestartPolicy, ChecksummedPublicationHead,
    FilesystemCatalogSnapshot, filesystem_initialization_namespace, filesystem_version_two_records,
    publication_head_decoder,
};
use crate::{RetentionHead, RetentionManifest, RetentionNamespaceDigest};

const HEAD_NAME: &str = "HEAD";

#[cfg(test)]
#[path = "filesystem_retention_snapshot_pinning_tests.rs"]
mod pinning_tests;

#[cfg(test)]
#[path = "filesystem_retention_snapshot_moving_error_tests.rs"]
mod moving_error_tests;

#[cfg(test)]
#[path = "filesystem_retention_snapshot_coordinate_tests.rs"]
mod coordinate_tests;

/// One consistent reader view: the catalog snapshot, the retention head, and
/// the manifest it selects, all observed under one shared reader fence.
///
/// The view holds the fence for its lifetime, so collection cannot delete the
/// roots or segments it names while it lives. Selected roots are read on
/// demand and verified against the manifest's digest before they are returned.
#[must_use]
pub struct FilesystemRetentionSnapshot {
    _fence: ReaderFence,
    roots: Dir,
    catalog: FilesystemCatalogSnapshot,
    retention: Option<ObservedRetentionState>,
}

pub(super) struct View {
    catalog: FilesystemCatalogSnapshot,
    retention: Option<ObservedRetentionState>,
}

pub(super) struct Source {
    root: Dir,
    retention: Dir,
    manifests: Dir,
    policy: CatalogRestartPolicy,
}

impl RetentionViewSource for Source {
    type View = Result<View, CatalogRestartError>;

    fn coordinates(&mut self) -> io::Result<RetentionViewCoordinates> {
        let catalog = filesystem_retention_current::read_exact_optional(
            &self.root,
            HEAD_NAME,
            publication_head_decoder::ENCODED_LENGTH,
        )?
        .map(|bytes| {
            ChecksummedPublicationHead::decode(&bytes)
                .map(|head| {
                    (
                        head.generation(),
                        head.catalog_length(),
                        head.catalog_digest(),
                    )
                })
                .map_err(|source| io::Error::new(io::ErrorKind::InvalidData, source))
        })
        .transpose()?;
        let retention = filesystem_retention_current::observe(&self.retention, &self.manifests)?
            .map(|state| *state.head());
        Ok(RetentionViewCoordinates { catalog, retention })
    }

    fn load(&mut self) -> io::Result<Self::View> {
        let catalog = crate::adapters::catalog_restart_loader::load_from_directory(
            &self.root,
            HEAD_NAME,
            self.policy,
        );
        let catalog = match catalog {
            Ok(catalog) => catalog,
            Err(source) => return Ok(Err(source)),
        };
        let retention = filesystem_retention_current::observe(&self.retention, &self.manifests)?;
        Ok(Ok(View { catalog, retention }))
    }
}

impl FilesystemRetentionSnapshot {
    /// Admits the root as version two, acquires the reader fence, and
    /// double-collects one consistent view within `limit` attempts.
    /// Admission requires the opened directory's restart-stable device and
    /// inode to match the jointly admitted migration records before fencing.
    ///
    /// The call takes no writer authority and mutates nothing. It may block
    /// while collection holds the fence exclusively.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemRetentionSnapshotError`](super::FilesystemRetentionSnapshotError)
    /// at the exact admission, fence, collection, or catalog refusal.
    /// A catalog admission result is returned only after both coordinate
    /// reads agree; a moving head discards that result and retries.
    pub fn load(
        store_root: &Path,
        policy: CatalogRestartPolicy,
        limit: ReaderAttemptLimit,
    ) -> Result<Self, Error> {
        Self::load_with(store_root, policy, limit, |source, limit| {
            collect_retention_view(source, limit)
                .map_err(|source| Error::View { source })?
                .map_err(|source| Error::Catalog { source })
        })
    }

    pub(super) fn load_with<E: From<Error>>(
        store_root: &Path,
        policy: CatalogRestartPolicy,
        limit: ReaderAttemptLimit,
        collect: impl FnOnce(&mut Source, ReaderAttemptLimit) -> Result<View, E>,
    ) -> Result<Self, E> {
        let root = Dir::open_ambient_dir(store_root, cap_std::ambient_authority())
            .map_err(|source| Error::Admission { source })?;
        filesystem_initialization_namespace::admit_version_two(&root)
            .map_err(|source| Error::Admission { source })?;
        let bound = filesystem_version_two_records::admit(&root)
            .map_err(|source| Error::Admission { source })?;
        let observed = root_identity(&root).map_err(|source| Error::Admission { source })?;
        require_root_identity(bound, observed).map_err(|source| Error::Admission {
            source: io::Error::new(io::ErrorKind::InvalidData, source),
        })?;
        let fence = ReaderFence::acquire(&root).map_err(|source| Error::Fence { source })?;
        let retention = root
            .open_dir_nofollow(pool_name::RETENTION)
            .map_err(|source| Error::Admission { source })?;
        let roots = retention
            .open_dir_nofollow(pool_name::ROOTS)
            .map_err(|source| Error::Admission { source })?;
        let manifests = retention
            .open_dir_nofollow(pool_name::MANIFESTS)
            .map_err(|source| Error::Admission { source })?;
        let mut source = Source {
            root,
            retention,
            manifests,
            policy,
        };
        let view = collect(&mut source, limit)?;
        Ok(Self {
            _fence: fence,
            roots,
            catalog: view.catalog,
            retention: view.retention,
        })
    }

    /// The catalog snapshot the view binds.
    pub const fn catalog(&self) -> &FilesystemCatalogSnapshot {
        &self.catalog
    }

    /// The published retention head, or `None` when no generation is published.
    #[must_use]
    pub fn retention_head(&self) -> Option<&RetentionHead> {
        self.retention.as_ref().map(ObservedRetentionState::head)
    }

    /// The manifest the retention head selects, or `None` when none is published.
    #[must_use]
    pub fn manifest(&self) -> Option<&RetentionManifest> {
        self.retention
            .as_ref()
            .map(ObservedRetentionState::manifest)
    }

    /// Reads and verifies the root the manifest selects for `namespace`.
    ///
    /// Returns `None` when the manifest names no root for the namespace. The
    /// pool entry is read without following links, bounded by the root
    /// format's maximum length, decoded, and required to carry exactly the
    /// generation and digest the manifest names.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemRetentionSnapshotError::Root`](super::FilesystemRetentionSnapshotError::Root)
    /// when the entry is absent, unreadable, or not the selected root.
    pub fn retained_root(
        &self,
        namespace: RetentionNamespaceDigest,
    ) -> Result<Option<Box<[u8]>>, Error> {
        let Some(manifest) = self.manifest() else {
            return Ok(None);
        };
        let entries = manifest.entries();
        let Some(entry) = entries
            .binary_search_by_key(&namespace, |entry| entry.namespace())
            .ok()
            .and_then(|index| entries.get(index).copied())
        else {
            return Ok(None);
        };
        let directory = self
            .roots
            .open_dir_nofollow(pool_name::namespace(namespace))
            .map_err(|source| Error::Root { source })?;
        let name = pool_name::root(entry.root_generation(), entry.root_digest());
        let length = directory
            .symlink_metadata(&name)
            .and_then(|metadata| {
                usize::try_from(metadata.len()).map_err(|_source| {
                    selected_refusal(RetentionSelectedRootRefusal::HostLength {
                        observed: metadata.len(),
                    })
                })
            })
            .map_err(|source| Error::Root { source })?;
        if length > root_header_decoder::MAXIMUM_ENCODED_LENGTH {
            return Err(Error::Root {
                source: selected_refusal(RetentionSelectedRootRefusal::Length {
                    maximum: u64::try_from(root_header_decoder::MAXIMUM_ENCODED_LENGTH).map_err(
                        |source| Error::Root {
                            source: io::Error::other(source),
                        },
                    )?,
                    observed: u64::try_from(length).map_err(|source| Error::Root {
                        source: io::Error::other(source),
                    })?,
                }),
            });
        }
        let bytes = match exact_record::read_exact_optional(&directory, &name, length) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => {
                return Err(Error::Root {
                    source: selected_refusal(RetentionSelectedRootRefusal::Absent),
                });
            }
            Err(ExactRecordError::Io(source)) => return Err(Error::Root { source }),
            Err(ExactRecordError::Refused(refusal)) => {
                return Err(Error::Root {
                    source: ExactRecordError::Refused(refusal).into_io(),
                });
            }
        };
        let root = AdmittedRetentionRoot::decode(&bytes).map_err(|source| Error::Root {
            source: io::Error::new(io::ErrorKind::InvalidData, source),
        })?;
        if root.digest() != entry.root_digest()
            || root.root().generation() != entry.root_generation()
        {
            return Err(Error::Root {
                source: selected_refusal(RetentionSelectedRootRefusal::Coordinate {
                    expected_generation: entry.root_generation(),
                    observed_generation: root.root().generation(),
                    expected_digest: entry.root_digest(),
                    observed_digest: root.digest(),
                }),
            });
        }
        Ok(Some(bytes.into_boxed_slice()))
    }
}

fn selected_refusal(refusal: RetentionSelectedRootRefusal) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, refusal)
}
