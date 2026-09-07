//! This module owns exact observation of the current filesystem retention state.

use std::io::{self, Read};

use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::fs::{Dir, OpenOptions};

use super::filesystem_retention_pool_name as pool_name;
use super::filesystem_retention_stage::invalid_data;
use super::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, ChecksummedRetentionHead,
    RetentionPublicationPreparation, RetentionTransitionDisposition,
};
use crate::RetentionGenerationExpectation;

const HEAD_LENGTH: usize = 144;

/// Exact bytes of one published retention head and the manifest it selects.
///
/// Both records were reopened without following links, bounded by their
/// declared lengths, decoded, and cross-checked: the manifest's canonical
/// digest equals the digest the head names. Callers decode the bytes with
/// [`ChecksummedRetentionHead`] and [`AdmittedRetentionManifest`] to plan the
/// next transition.
#[must_use]
#[derive(Debug)]
pub struct ObservedRetentionState {
    head: Box<[u8]>,
    manifest: Box<[u8]>,
}

impl ObservedRetentionState {
    /// Returns the exact 144 published head bytes.
    pub const fn head_bytes(&self) -> &[u8] {
        &self.head
    }

    /// Returns the exact bytes of the manifest the head selects.
    pub const fn manifest_bytes(&self) -> &[u8] {
        &self.manifest
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
        .map_err(|_source| invalid_data("current retention head refused admission"))?;
    let selected = decoded.head();
    let length = usize::try_from(selected.manifest_length().get())
        .map_err(|_source| invalid_data("current manifest length exceeded usize"))?;
    let name = pool_name::manifest(selected.generation(), selected.manifest_digest());
    let manifest = read_exact_optional(manifests, &name, length)?
        .ok_or_else(|| invalid_data("current retention head names an absent manifest"))?;
    let admitted = AdmittedRetentionManifest::decode(&manifest)
        .map_err(|_source| invalid_data("current retention manifest refused admission"))?;
    if admitted.digest() != selected.manifest_digest()
        || admitted.manifest().generation() != selected.generation()
    {
        return Err(invalid_data(
            "current retention manifest disagreed with its head",
        ));
    }
    Ok(Some(ObservedRetentionState { head, manifest }))
}

/// Compares one preparation against the observed current state.
///
/// An absent head admits only an `Absent` expectation. A present head that
/// equals the prepared successor is `AlreadyCommitted`. Otherwise the head
/// must be the exact predecessor the prepared successor names, or the
/// candidate is superseded and refuses.
pub(super) fn disposition(
    preparation: &RetentionPublicationPreparation<'_>,
    current: Option<&ObservedRetentionState>,
) -> io::Result<RetentionTransitionDisposition> {
    let Some(current) = current else {
        return match preparation.expected() {
            RetentionGenerationExpectation::Absent => require_initial_publication(preparation),
            RetentionGenerationExpectation::Current(_) => Err(invalid_data(
                "expected a current retention generation but no head is published",
            )),
        };
    };
    let head = ChecksummedRetentionHead::decode(current.head_bytes())
        .map_err(|_source| invalid_data("observed retention head refused admission"))?;
    let head = head.head();
    let committed = (
        preparation.liveness_generation(),
        preparation.manifest_digest(),
    );
    if (head.generation(), head.manifest_digest()) == committed {
        return Ok(RetentionTransitionDisposition::AlreadyCommitted);
    }
    let publication = preparation.publication().ok_or_else(|| {
        invalid_data("already-committed retry is stale: another successor is current")
    })?;
    let prepared = ChecksummedRetentionHead::decode(publication.head().encoded())
        .map_err(|_source| invalid_data("prepared retention head refused admission"))?;
    let expected_generation = head
        .generation()
        .successor()
        .map_err(|_source| invalid_data("current liveness generation cannot advance"))?;
    if prepared.head().predecessor() == Some(head.manifest_digest())
        && prepared.head().generation() == expected_generation
    {
        Ok(RetentionTransitionDisposition::Publish)
    } else {
        Err(invalid_data(
            "current retention head is not the prepared predecessor; the candidate is superseded",
        ))
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
    let manifest = AdmittedRetentionManifest::decode(current.manifest_bytes())
        .map_err(|_source| invalid_data("observed retention manifest refused admission"))?;
    let namespace = candidate.root().namespace().digest();
    let entries = manifest.manifest().entries();
    let entry = entries
        .binary_search_by_key(&namespace, |entry| entry.namespace())
        .ok()
        .and_then(|index| entries.get(index).copied())
        .ok_or_else(|| {
            invalid_data("committed manifest does not select the candidate namespace")
        })?;
    if entry.root_generation() != candidate.root().generation()
        || entry.root_digest() != candidate.digest()
    {
        return Err(invalid_data(
            "committed manifest selects a different root for the candidate namespace",
        ));
    }
    let directory = roots
        .open_dir_nofollow(pool_name::namespace(namespace))
        .map_err(|_source| invalid_data("committed root namespace directory is unavailable"))?;
    let name = pool_name::root(candidate.root().generation(), candidate.digest());
    let observed = read_exact_optional(&directory, &name, candidate.encoded().len())?
        .ok_or_else(|| invalid_data("committed root pool entry is absent"))?;
    if observed.as_ref() == candidate.encoded() {
        Ok(())
    } else {
        Err(invalid_data("committed root pool entry bytes disagreed"))
    }
}

/// The empty retention state admits only a generation-one head with no predecessor.
fn require_initial_publication(
    preparation: &RetentionPublicationPreparation<'_>,
) -> io::Result<RetentionTransitionDisposition> {
    let publication = preparation
        .publication()
        .ok_or_else(|| invalid_data("already-committed retry against an absent retention head"))?;
    let prepared = ChecksummedRetentionHead::decode(publication.head().encoded())
        .map_err(|_source| invalid_data("prepared retention head refused admission"))?;
    if prepared.head().generation() == crate::LivenessGeneration::INITIAL
        && prepared.head().predecessor().is_none()
    {
        Ok(RetentionTransitionDisposition::Publish)
    } else {
        Err(invalid_data(
            "absent retention head admits only an initial publication with no predecessor",
        ))
    }
}

fn read_exact_optional(
    directory: &Dir,
    name: &str,
    length: usize,
) -> io::Result<Option<Box<[u8]>>> {
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No).nonblock(true);
    let mut file = match directory.open_with(name, &options) {
        Ok(file) => file,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(source),
    };
    let expected_length = u64::try_from(length)
        .map_err(|_source| invalid_data("retention record length exceeded u64"))?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() != expected_length {
        return Err(invalid_data("retention record kind or length disagreed"));
    }
    let mut bytes = vec![0_u8; length];
    file.read_exact(&mut bytes)?;
    let mut trailing = [0_u8; 1];
    if file.read(&mut trailing)? != 0 {
        return Err(invalid_data("retention record carried trailing bytes"));
    }
    Ok(Some(bytes.into_boxed_slice()))
}
