//! This module owns restart observation of the retention stages and pools.

use std::io::{self, Read};

use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

use super::filesystem_retention_current::{self, ObservedRetentionState};
use super::filesystem_retention_pool_name as pool_name;
use super::{
    RetentionPoolEntryObservation as Pool, RetentionPoolObservations, RetentionRecoveryEvidence,
    RetentionStageAssessment, RetentionStageAssessments, assess_head_stage, assess_manifest_stage,
    assess_root_stage, head_decoder, manifest_header_decoder, root_header_decoder,
};
use crate::adapters::filesystem_exact_record::{
    self as exact_record, EntryIdentity, ExactRecordError,
};

/// The exact bytes and entry identity of one retained stage.
pub(super) struct StageBytes {
    pub(super) bytes: Box<[u8]>,
    pub(super) identity: EntryIdentity,
}

/// Everything restart read under writer authority before planning recovery.
pub(super) struct RetentionRecoveryObservation {
    current: Option<ObservedRetentionState>,
    root: Option<StageBytes>,
    manifest: Option<StageBytes>,
    head: Option<StageBytes>,
    pools: RetentionPoolObservations,
}

impl RetentionRecoveryObservation {
    /// Reads the current state, the three stages, and the pool entries the
    /// complete stages name. Performs no mutation.
    pub(super) fn observe(retention: &Dir, roots: &Dir, manifests: &Dir) -> io::Result<Self> {
        let current = filesystem_retention_current::observe(retention, manifests)?;
        let root = read_stage(
            retention,
            pool_name::ROOT_STAGE,
            root_header_decoder::MAXIMUM_ENCODED_LENGTH,
        )?;
        let manifest = read_stage(
            retention,
            pool_name::MANIFEST_STAGE,
            manifest_header_decoder::maximum_encoded_length()
                .map_err(|source| io::Error::new(io::ErrorKind::InvalidData, source))?,
        )?;
        let head = read_stage(
            retention,
            pool_name::HEAD_STAGE,
            head_decoder::ENCODED_LENGTH,
        )?;
        let root_pool = match root
            .as_ref()
            .map(|stage| (assess_root_stage(Some(&stage.bytes)), stage.identity))
        {
            Some((RetentionStageAssessment::Complete(admitted), identity)) => {
                let namespace = pool_name::namespace(admitted.root().namespace().digest());
                let name = pool_name::root(admitted.root().generation(), admitted.digest());
                match roots.open_dir_nofollow(namespace) {
                    Ok(directory) => pool_entry(&directory, &name, admitted.encoded(), identity)?,
                    Err(source) if source.kind() == io::ErrorKind::NotFound => Pool::Absent,
                    Err(source) => return Err(source),
                }
            }
            _ => Pool::Absent,
        };
        let manifest_pool = match manifest
            .as_ref()
            .map(|stage| (assess_manifest_stage(Some(&stage.bytes)), stage.identity))
        {
            Some((RetentionStageAssessment::Complete(admitted), identity)) => {
                let name = pool_name::manifest(admitted.manifest().generation(), admitted.digest());
                pool_entry(manifests, &name, admitted.encoded(), identity)?
            }
            _ => Pool::Absent,
        };
        Ok(Self {
            current,
            root,
            manifest,
            head,
            pools: RetentionPoolObservations {
                root: root_pool,
                manifest: manifest_pool,
            },
        })
    }

    /// The pure evidence recovery plans from.
    pub(super) fn evidence(&self) -> RetentionRecoveryEvidence<'_, '_> {
        RetentionRecoveryEvidence::new(
            self.current.as_ref(),
            RetentionStageAssessments {
                root: assess_root_stage(self.root.as_ref().map(|stage| &*stage.bytes)),
                manifest: assess_manifest_stage(self.manifest.as_ref().map(|stage| &*stage.bytes)),
                head: assess_head_stage(self.head.as_ref().map(|stage| &*stage.bytes)),
            },
            self.pools,
        )
    }

    pub(super) const fn root(&self) -> Option<&StageBytes> {
        self.root.as_ref()
    }

    pub(super) const fn manifest(&self) -> Option<&StageBytes> {
        self.manifest.as_ref()
    }

    pub(super) const fn head(&self) -> Option<&StageBytes> {
        self.head.as_ref()
    }
}

/// Reads a stage's complete bytes up to one byte past `bound`.
///
/// A stage longer than its format's maximum is returned in full up to that
/// point so assessment classifies it as corrupt rather than truncated.
fn read_stage(retention: &Dir, name: &str, bound: usize) -> io::Result<Option<StageBytes>> {
    let mut file = match exact_record::open_read(retention, name) {
        Ok(file) => file,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(source),
    };
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "retained retention stage is not a regular file",
        ));
    }
    let identity = EntryIdentity::from(&metadata);
    let limit = bound
        .checked_add(1)
        .and_then(|limit| u64::try_from(limit).ok())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "stage bound overflowed"))?;
    let mut bytes = Vec::new();
    file.by_ref().take(limit).read_to_end(&mut bytes)?;
    Ok(Some(StageBytes {
        bytes: bytes.into_boxed_slice(),
        identity,
    }))
}

/// Whether the pool entry names the exact retained stage inode and bytes.
fn pool_entry(
    directory: &Dir,
    name: &str,
    expected: &[u8],
    identity: EntryIdentity,
) -> io::Result<Pool> {
    match exact_record::verify_named(directory, name, expected, identity) {
        Ok(()) => Ok(Pool::Identical),
        Err(ExactRecordError::Io(source)) if source.kind() == io::ErrorKind::NotFound => {
            Ok(Pool::Absent)
        }
        Err(ExactRecordError::Refused(_)) => Ok(Pool::Different),
        Err(ExactRecordError::Io(source)) => Err(source),
    }
}
