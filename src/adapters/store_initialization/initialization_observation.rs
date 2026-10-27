//! This module owns the mutation-free classification of an initialization root.
//!
//! The classifier decides which recoverable state a root is in, or refuses it.
//! It opens nothing for writing and follows no links. Its result authorizes
//! effects only when it runs under writer authority.

use std::ffi::OsString;
use std::io::{self, Read};
use std::path::PathBuf;

use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

use super::{
    StoreInitializationDirectory as Directory, StoreInitializationError as Error,
    StoreInitializationPhase as Phase, StoreInitializationPrefix as Prefix,
    StoreInitializationRefusal as Refusal, initialization_namespace as names,
};
use crate::adapters::filesystem_exact_record::{self as exact_record, EntryIdentity};
use crate::adapters::filesystem_namespace_refusal::FilesystemEntryKind;
use crate::adapters::{CanonicalStoreFormatMarker, FORMAT_MARKER_LENGTH};

/// Classifies `root` as exactly one lawful prefix, or refuses it.
///
/// # Errors
///
/// Returns [`Error::Refused`] for every root that is not a recoverable
/// prefix, including a committed one, and [`Error::Io`] at `ObservePrefix`
/// for a failed observation.
pub(super) fn observe(root: &Dir) -> Result<Prefix, Error> {
    let present = root_names(root)?;
    let has = |name: &str| present.iter().any(|entry| entry == name);
    if has(names::MARKER) {
        return if has(names::MARKER_STAGE) {
            observe_linked(root)
        } else {
            Err(Error::refused(Refusal::AlreadyInitialized))
        };
    }
    for committed_only in [names::HEAD, names::NEXT_HEAD] {
        if has(committed_only) {
            return not_a_prefix(committed_only);
        }
    }
    if !has(names::WRITER_LOCK) {
        return refuse_other(present.first().cloned(), Prefix::Empty);
    }
    require_empty_lock(root, names::WRITER_LOCK)?;
    observe_after_lock(root, &present)
}

fn observe_after_lock(root: &Dir, present: &[OsString]) -> Result<Prefix, Error> {
    let has = |name: &str| present.iter().any(|entry| entry == name);
    let others = |allowed: &[&str]| {
        present
            .iter()
            .find(|entry| !allowed.iter().any(|name| *entry == *name))
            .cloned()
    };
    if !has(names::MARKER_STAGE) {
        return refuse_other(others(&[names::WRITER_LOCK]), Prefix::Locked);
    }
    let length = observe_marker_stage(root)?;
    if length < FORMAT_MARKER_LENGTH {
        let allowed = [names::WRITER_LOCK, names::MARKER_STAGE];
        return refuse_other(others(&allowed), Prefix::IncompleteMarkerStage { length });
    }
    if !has(names::READER_LOCK) {
        let allowed = [names::WRITER_LOCK, names::MARKER_STAGE];
        return refuse_other(others(&allowed), Prefix::MarkerStaged);
    }
    require_empty_lock(root, names::READER_LOCK)?;
    observe_directories(root)
}

fn refuse_other(other: Option<OsString>, prefix: Prefix) -> Result<Prefix, Error> {
    other.map_or(Ok(prefix), |entry| {
        Err(Error::refused(Refusal::NotAPrefix {
            entry: PathBuf::from(entry),
        }))
    })
}

/// Requires the present directories to be exactly a creation-order prefix.
fn observe_directories(root: &Dir) -> Result<Prefix, Error> {
    let mut last = None;
    for directory in Directory::ALL {
        let present = directory_present(root, directory)?;
        match (present, last) {
            (true, previous) if previous == predecessor(directory) => {
                require_membership(root, directory)?;
                last = Some(directory);
            }
            (true, _) => return not_a_prefix(directory.path()),
            (false, _) => {}
        }
    }
    Ok(last.map_or(Prefix::ReaderFenced, |last| Prefix::Directories { last }))
}

fn predecessor(directory: Directory) -> Option<Directory> {
    let index = Directory::ALL
        .iter()
        .position(|candidate| *candidate == directory)?;
    index
        .checked_sub(1)
        .and_then(|previous| Directory::ALL.get(previous).copied())
}

fn directory_present(root: &Dir, directory: Directory) -> Result<bool, Error> {
    if let Some(parent) = directory.parent()
        && !directory_present(root, parent)?
    {
        return Ok(false);
    }
    match root.symlink_metadata(directory.path()) {
        Ok(metadata) => {
            let observed = FilesystemEntryKind::observed(metadata.file_type());
            if observed == FilesystemEntryKind::Directory {
                Ok(true)
            } else {
                Err(Error::refused(Refusal::WrongKind {
                    entry: directory.path(),
                    expected: FilesystemEntryKind::Directory,
                    observed,
                }))
            }
        }
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(source) => Err(observation(source)),
    }
}

/// Requires a present directory to hold only its canonical children.
fn require_membership(root: &Dir, directory: Directory) -> Result<(), Error> {
    let opened = root
        .open_dir_nofollow(directory.path())
        .map_err(observation)?;
    for entry in opened.entries().map_err(observation)? {
        let name = entry.map_err(observation)?.file_name();
        let canonical = directory
            .children()
            .iter()
            .any(|child| name == child.name());
        if !canonical {
            let mut path = PathBuf::from(directory.path());
            path.push(name);
            return Err(Error::refused(Refusal::NotAPrefix { entry: path }));
        }
    }
    Ok(())
}

/// Returns the stage length after proving it is a canonical byte prefix.
fn observe_marker_stage(root: &Dir) -> Result<usize, Error> {
    let entry = root
        .symlink_metadata(names::MARKER_STAGE)
        .map_err(observation)?;
    require_kind(names::MARKER_STAGE, &entry, FilesystemEntryKind::File)?;
    let file = exact_record::open_regular(root, names::MARKER_STAGE).map_err(observation)?;
    let metadata = file.metadata().map_err(observation)?;
    require_kind(names::MARKER_STAGE, &metadata, FilesystemEntryKind::File)?;
    let mismatch = || {
        Error::refused(Refusal::MarkerStageMismatch {
            expected_length: FORMAT_MARKER_LENGTH,
            observed_length: metadata.len(),
        })
    };
    let length = usize::try_from(metadata.len()).map_err(|_| mismatch())?;
    if length > FORMAT_MARKER_LENGTH {
        return Err(mismatch());
    }
    let bound = u64::try_from(FORMAT_MARKER_LENGTH).map_err(|_| mismatch())?;
    let mut bytes = Vec::with_capacity(FORMAT_MARKER_LENGTH);
    file.take(bound.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(observation)?;
    let canonical = CanonicalStoreFormatMarker::version_two();
    if bytes.len() == length && canonical.encoded().starts_with(&bytes) {
        Ok(length)
    } else {
        Err(mismatch())
    }
}

/// Classifies `FORMAT` beside `FORMAT.next`: one canonical inode, committed namespace.
fn observe_linked(root: &Dir) -> Result<Prefix, Error> {
    let canonical = CanonicalStoreFormatMarker::version_two();
    let stage = exact_record::open_regular(root, names::MARKER_STAGE).map_err(observation)?;
    let identity = EntryIdentity::of_file(&stage).map_err(observation)?;
    for name in [names::MARKER_STAGE, names::MARKER] {
        exact_record::verify_named(root, name, canonical.encoded(), identity)
            .map_err(|_| Error::refused(Refusal::MarkerLinkMismatch))?;
    }
    names::admit_linked_namespace(root).map_err(observation)?;
    Ok(Prefix::MarkerLinked)
}

fn require_empty_lock(root: &Dir, name: &'static str) -> Result<(), Error> {
    let metadata = root.symlink_metadata(name).map_err(observation)?;
    require_kind(name, &metadata, FilesystemEntryKind::File)?;
    if metadata.len() == 0 {
        Ok(())
    } else {
        Err(Error::refused(Refusal::NonEmptyLockFile {
            entry: name,
            observed_length: metadata.len(),
        }))
    }
}

fn require_kind(
    entry: &'static str,
    metadata: &cap_std::fs::Metadata,
    expected: FilesystemEntryKind,
) -> Result<(), Error> {
    let observed = FilesystemEntryKind::observed(metadata.file_type());
    if observed == expected {
        Ok(())
    } else {
        Err(Error::refused(Refusal::WrongKind {
            entry,
            expected,
            observed,
        }))
    }
}

/// Lists root entry names in sorted order, refusing any unknown name.
fn root_names(root: &Dir) -> Result<Vec<OsString>, Error> {
    let mut present = Vec::new();
    for entry in root.entries().map_err(observation)? {
        let name = entry.map_err(observation)?.file_name();
        if !names::is_root_name(&name) {
            return Err(Error::refused(Refusal::NotAPrefix {
                entry: PathBuf::from(name),
            }));
        }
        present.push(name);
    }
    present.sort();
    Ok(present)
}

fn not_a_prefix<T>(entry: &str) -> Result<T, Error> {
    Err(Error::refused(Refusal::NotAPrefix {
        entry: PathBuf::from(entry),
    }))
}

const fn observation(source: io::Error) -> Error {
    Error::io(Phase::ObservePrefix, source)
}
