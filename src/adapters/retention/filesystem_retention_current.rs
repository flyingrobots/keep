//! This module owns exact observation of the current filesystem retention state.

use std::io;

use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

use super::filesystem_retention_pool_name as pool_name;
use super::root_header_decoder;
use super::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, ChecksummedRetentionHead,
    RetentionCurrentStateRefusal, RetentionPublicationPreparation, RetentionTransitionDisposition,
};
use crate::adapters::filesystem_exact_record::{
    self as exact_record, ExactRecordError, ExactRecordRefusal,
};
use crate::{RetentionGenerationExpectation, RetentionHead, RetentionManifest};

const HEAD_LENGTH: usize = super::head_decoder::ENCODED_LENGTH;

/// One published retention head and the manifest it selects, bytes and values.
///
/// Both records were reopened without following links, bounded by their
/// declared lengths, decoded exactly once, and cross-checked: the manifest's
/// canonical digest, generation, and predecessor equal what the head names.
/// Callers plan the next transition from the decoded values or re-admit the
/// exact bytes with [`ChecksummedRetentionHead`] and
/// [`AdmittedRetentionManifest`].
#[must_use]
#[derive(Debug)]
pub struct ObservedRetentionState {
    head: Box<[u8]>,
    manifest: Box<[u8]>,
    decoded_head: RetentionHead,
    decoded_manifest: RetentionManifest,
}

impl ObservedRetentionState {
    /// Returns the exact published head bytes.
    pub const fn head_bytes(&self) -> &[u8] {
        &self.head
    }

    /// Returns the exact bytes of the manifest the head selects.
    pub const fn manifest_bytes(&self) -> &[u8] {
        &self.manifest
    }

    /// Returns the decoded head coordinate.
    pub const fn head(&self) -> &RetentionHead {
        &self.decoded_head
    }

    /// Returns the decoded manifest the head selects.
    pub const fn manifest(&self) -> &RetentionManifest {
        &self.decoded_manifest
    }
}

/// The verified relationship between one preparation and the observed state.
#[derive(Clone, Copy)]
pub(super) enum ObservedDisposition<'state> {
    /// The observed head already names the prepared successor.
    Committed(&'state ObservedRetentionState),
    /// The prepared successor advances the observed state.
    Publish,
}

impl ObservedDisposition<'_> {
    pub(super) const fn transition(self) -> RetentionTransitionDisposition {
        match self {
            Self::Committed(_) => RetentionTransitionDisposition::AlreadyCommitted,
            Self::Publish => RetentionTransitionDisposition::Publish,
        }
    }
}

/// Reads and cross-verifies `retention/HEAD` and its selected manifest.
///
/// Returns `None` only when no head is published. A present head that does
/// not decode, or whose manifest is missing, wrong-length, or names another
/// digest, refuses.
pub(super) fn observe(
    retention: &Dir,
    manifests: &Dir,
) -> io::Result<Option<ObservedRetentionState>> {
    let Some(head) = read_exact_optional(retention, pool_name::HEAD, HEAD_LENGTH)? else {
        return Ok(None);
    };
    let decoded = ChecksummedRetentionHead::decode(&head)
        .map_err(|source| RetentionCurrentStateRefusal::HeadRefused { source }.into_io())?;
    let selected = decoded.head();
    let length = usize::try_from(selected.manifest_length().get())
        .map_err(|_source| RetentionCurrentStateRefusal::RecordLengthOverflow.into_io())?;
    let name = pool_name::manifest(selected.generation(), selected.manifest_digest());
    let manifest = read_exact_optional(manifests, &name, length)?
        .ok_or_else(|| RetentionCurrentStateRefusal::ManifestAbsent.into_io())?;
    let admitted = AdmittedRetentionManifest::decode(&manifest)
        .map_err(|source| RetentionCurrentStateRefusal::ManifestRefused { source }.into_io())?;
    if admitted.digest() != selected.manifest_digest()
        || admitted.manifest().generation() != selected.generation()
    {
        return Err(RetentionCurrentStateRefusal::ManifestDisagreed.into_io());
    }
    if admitted.manifest().predecessor() != selected.predecessor() {
        return Err(RetentionCurrentStateRefusal::HeadPredecessorDisagreed.into_io());
    }
    let decoded_head = *selected;
    let decoded_manifest = admitted.manifest().clone();
    Ok(Some(ObservedRetentionState {
        head,
        manifest,
        decoded_head,
        decoded_manifest,
    }))
}

/// Compares one preparation against the observed current state.
///
/// An absent head admits only an `Absent` expectation. A present head that
/// equals the prepared successor is `AlreadyCommitted`. Otherwise the head
/// must be the exact predecessor the prepared successor names, or the
/// candidate is superseded and refuses.
pub(super) fn disposition<'state>(
    preparation: &RetentionPublicationPreparation<'_>,
    current: Option<&'state ObservedRetentionState>,
) -> io::Result<ObservedDisposition<'state>> {
    let Some(current) = current else {
        return match preparation.expected() {
            RetentionGenerationExpectation::Absent => {
                require_initial_publication(preparation).map(|()| ObservedDisposition::Publish)
            }
            RetentionGenerationExpectation::Current(_) => {
                Err(RetentionCurrentStateRefusal::ExpectedCurrentOverAbsentHead.into_io())
            }
        };
    };
    let head = current.head();
    let committed = (
        preparation.liveness_generation(),
        preparation.manifest_digest(),
    );
    if (head.generation(), head.manifest_digest()) == committed {
        return Ok(ObservedDisposition::Committed(current));
    }
    let publication = preparation
        .publication()
        .ok_or_else(|| RetentionCurrentStateRefusal::StaleCommittedRetry.into_io())?;
    let prepared = ChecksummedRetentionHead::decode(publication.head().encoded())
        .map_err(|source| RetentionCurrentStateRefusal::PreparedHeadRefused { source }.into_io())?;
    let expected_generation = head
        .generation()
        .successor()
        .map_err(|_source| RetentionCurrentStateRefusal::LivenessExhausted.into_io())?;
    if prepared.head().predecessor() == Some(head.manifest_digest())
        && prepared.head().generation() == expected_generation
    {
        Ok(ObservedDisposition::Publish)
    } else {
        Err(RetentionCurrentStateRefusal::Superseded {
            current_generation: head.generation(),
            current_digest: head.manifest_digest(),
        }
        .into_io())
    }
}

/// Reopens the evidence behind an `AlreadyCommitted` disposition.
///
/// The observed manifest must select `candidate`'s namespace at exactly its
/// generation and digest, and the immutable root-pool entry must reopen with
/// exactly `candidate`'s bytes. A head that merely agrees with its manifest is
/// not proof that the claimed root is still available.
pub(super) fn verify_committed(
    roots: &Dir,
    current: &ObservedRetentionState,
    candidate: &AdmittedRetentionRoot<'_>,
) -> io::Result<()> {
    let namespace = candidate.root().namespace().digest();
    let entries = current.manifest().entries();
    let entry = entries
        .binary_search_by_key(&namespace, |entry| entry.namespace())
        .ok()
        .and_then(|index| entries.get(index).copied())
        .ok_or_else(|| RetentionCurrentStateRefusal::CommittedSelectionMissing.into_io())?;
    if entry.root_generation() != candidate.root().generation()
        || entry.root_digest() != candidate.digest()
    {
        return Err(RetentionCurrentStateRefusal::CommittedSelectionMismatch.into_io());
    }
    let directory = roots
        .open_dir_nofollow(pool_name::namespace(namespace))
        .map_err(|_source| RetentionCurrentStateRefusal::CommittedNamespaceUnavailable.into_io())?;
    let name = pool_name::root(candidate.root().generation(), candidate.digest());
    let observed = read_exact_optional(&directory, &name, candidate.encoded().len())?
        .ok_or_else(|| RetentionCurrentStateRefusal::CommittedRootAbsent.into_io())?;
    if observed.as_ref() == candidate.encoded() {
        Ok(())
    } else {
        Err(RetentionCurrentStateRefusal::CommittedRootChanged.into_io())
    }
}

/// Reopens the predecessor root a `Current(_)` successor claims to advance.
///
/// The observed manifest must select the candidate's namespace, the candidate
/// must name that selection as its predecessor, and the selection's root pool
/// entry must reopen and decode to exactly that generation and digest. A
/// namespace directory alone is not proof the predecessor is available.
pub(super) fn verify_predecessor(
    roots: &Dir,
    current: &ObservedRetentionState,
    candidate: &AdmittedRetentionRoot<'_>,
) -> io::Result<()> {
    let namespace = candidate.root().namespace().digest();
    let entries = current.manifest().entries();
    let entry = entries
        .binary_search_by_key(&namespace, |entry| entry.namespace())
        .ok()
        .and_then(|index| entries.get(index).copied())
        .ok_or_else(|| RetentionCurrentStateRefusal::CommittedSelectionMissing.into_io())?;
    if candidate.root().predecessor() != Some(entry.root_digest()) {
        return Err(RetentionCurrentStateRefusal::PredecessorMismatch.into_io());
    }
    let directory = roots
        .open_dir_nofollow(pool_name::namespace(namespace))
        .map_err(|_source| RetentionCurrentStateRefusal::CommittedNamespaceUnavailable.into_io())?;
    let name = pool_name::root(entry.root_generation(), entry.root_digest());
    let length = match directory.symlink_metadata(&name) {
        Ok(metadata) => usize::try_from(metadata.len())
            .map_err(|_source| RetentionCurrentStateRefusal::RecordLengthOverflow.into_io())?,
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            return Err(RetentionCurrentStateRefusal::PredecessorRootAbsent.into_io());
        }
        Err(source) => return Err(source),
    };
    if length > root_header_decoder::MAXIMUM_ENCODED_LENGTH {
        return Err(RetentionCurrentStateRefusal::PredecessorRootAbsent.into_io());
    }
    let bytes = read_exact_optional(&directory, &name, length)?
        .ok_or_else(|| RetentionCurrentStateRefusal::PredecessorRootAbsent.into_io())?;
    let predecessor = AdmittedRetentionRoot::decode(&bytes)
        .map_err(|_source| RetentionCurrentStateRefusal::PredecessorRootChanged.into_io())?;
    if predecessor.digest() == entry.root_digest()
        && predecessor.root().generation() == entry.root_generation()
    {
        Ok(())
    } else {
        Err(RetentionCurrentStateRefusal::PredecessorRootChanged.into_io())
    }
}

/// The empty retention state admits only a generation-one head with no predecessor.
fn require_initial_publication(
    preparation: &RetentionPublicationPreparation<'_>,
) -> io::Result<()> {
    let publication = preparation
        .publication()
        .ok_or_else(|| RetentionCurrentStateRefusal::CommittedRetryOverAbsentHead.into_io())?;
    let prepared = ChecksummedRetentionHead::decode(publication.head().encoded())
        .map_err(|source| RetentionCurrentStateRefusal::PreparedHeadRefused { source }.into_io())?;
    if prepared.head().generation() == crate::LivenessGeneration::INITIAL
        && prepared.head().predecessor().is_none()
    {
        Ok(())
    } else {
        Err(RetentionCurrentStateRefusal::NonInitialOverAbsentHead.into_io())
    }
}

/// Reads one optional exact record, mapping shared refusals onto this protocol's.
pub(super) fn read_exact_optional(
    directory: &Dir,
    name: &str,
    length: usize,
) -> io::Result<Option<Box<[u8]>>> {
    match exact_record::read_exact_optional(directory, name, length) {
        Ok(bytes) => Ok(bytes.map(Vec::into_boxed_slice)),
        Err(ExactRecordError::Io(source)) => Err(source),
        Err(ExactRecordError::Refused(refusal)) => Err(match refusal {
            ExactRecordRefusal::LengthOverflow => {
                RetentionCurrentStateRefusal::RecordLengthOverflow
            }
            ExactRecordRefusal::TrailingBytes => RetentionCurrentStateRefusal::RecordTrailingBytes,
            ExactRecordRefusal::KindOrLength
            | ExactRecordRefusal::KindLengthOrIdentity
            | ExactRecordRefusal::Bytes
            | ExactRecordRefusal::RemainedVisible => {
                RetentionCurrentStateRefusal::RecordKindOrLength
            }
        }
        .into_io()),
    }
}
