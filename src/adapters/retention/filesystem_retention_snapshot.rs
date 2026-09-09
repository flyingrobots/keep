//! This module owns one fenced, double-collected reader view of a version-two store.

use std::io;
use std::path::{Path, PathBuf};

use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

use super::filesystem_retention_current::{self, ObservedRetentionState};
use super::filesystem_retention_pool_name as pool_name;
use super::{
    AdmittedRetentionRoot, FilesystemRetentionSnapshotError as Error, ReaderAttemptLimit,
    ReaderFence, RetentionViewCoordinates, RetentionViewSource, collect_retention_view,
    root_header_decoder,
};
use crate::adapters::filesystem_exact_record::{self as exact_record, ExactRecordError};
use crate::adapters::{
    CatalogRestartPolicy, ChecksummedPublicationHead, FilesystemCatalogSnapshot,
    filesystem_initialization_namespace, filesystem_version_two_records, publication_head_decoder,
};
use crate::{RetentionHead, RetentionManifest, RetentionNamespaceDigest};

const HEAD_NAME: &str = "HEAD";

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

struct View {
    catalog: FilesystemCatalogSnapshot,
    retention: Option<ObservedRetentionState>,
}

struct Source {
    root: Dir,
    retention: Dir,
    manifests: Dir,
    store_root: PathBuf,
    policy: CatalogRestartPolicy,
}

impl RetentionViewSource for Source {
    type View = View;

    fn coordinates(&mut self) -> io::Result<RetentionViewCoordinates> {
        let catalog = filesystem_retention_current::read_exact_optional(
            &self.root,
            HEAD_NAME,
            publication_head_decoder::ENCODED_LENGTH,
        )?
        .map(|bytes| {
            ChecksummedPublicationHead::decode(&bytes)
                .map(|head| (head.generation(), head.catalog_digest()))
                .map_err(|source| io::Error::new(io::ErrorKind::InvalidData, source))
        })
        .transpose()?;
        let retention = filesystem_retention_current::observe(&self.retention, &self.manifests)?
            .map(|state| (state.head().generation(), state.head().manifest_digest()));
        Ok(RetentionViewCoordinates { catalog, retention })
    }

    fn load(&mut self) -> io::Result<View> {
        let catalog = FilesystemCatalogSnapshot::load(&self.store_root, self.policy)
            .map_err(|source| io::Error::new(io::ErrorKind::InvalidData, source))?;
        let retention = filesystem_retention_current::observe(&self.retention, &self.manifests)?;
        Ok(View { catalog, retention })
    }
}

impl FilesystemRetentionSnapshot {
    /// Admits the root as version two, acquires the reader fence, and
    /// double-collects one consistent view within `limit` attempts.
    ///
    /// The call takes no writer authority and mutates nothing. It may block
    /// while collection holds the fence exclusively.
    ///
    /// # Errors
    ///
    /// Returns [`FilesystemRetentionSnapshotError`](super::FilesystemRetentionSnapshotError)
    /// at the exact admission, fence, collection, or catalog refusal.
    pub fn load(
        store_root: &Path,
        policy: CatalogRestartPolicy,
        limit: ReaderAttemptLimit,
    ) -> Result<Self, Error> {
        let root = Dir::open_ambient_dir(store_root, cap_std::ambient_authority())
            .map_err(|source| Error::Admission { source })?;
        filesystem_initialization_namespace::admit_version_two(&root)
            .map_err(|source| Error::Admission { source })?;
        let _bound = filesystem_version_two_records::admit(&root)
            .map_err(|source| Error::Admission { source })?;
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
            store_root: store_root.to_path_buf(),
            policy,
        };
        let view =
            collect_retention_view(&mut source, limit).map_err(|source| Error::View { source })?;
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
                usize::try_from(metadata.len()).map_err(|_source| invalid("root length overflow"))
            })
            .map_err(|source| Error::Root { source })?;
        if length > root_header_decoder::MAXIMUM_ENCODED_LENGTH {
            return Err(Error::Root {
                source: invalid("selected root exceeds the format bound"),
            });
        }
        let bytes = match exact_record::read_exact_optional(&directory, &name, length) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => {
                return Err(Error::Root {
                    source: invalid("selected root is absent"),
                });
            }
            Err(ExactRecordError::Io(source)) => return Err(Error::Root { source }),
            Err(ExactRecordError::Refused(refusal)) => {
                return Err(Error::Root {
                    source: invalid_string(format!("selected root refused: {refusal}")),
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
                source: invalid("selected root does not decode to the manifest's selection"),
            });
        }
        Ok(Some(bytes.into_boxed_slice()))
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn invalid_string(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
