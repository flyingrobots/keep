//! This module owns the ADR-0010 law for a copy torn by a concurrent writer.
//!
//! A copy taken while a writer publishes can hold a mix of the states before
//! and after that publication, and a partly copied file. These laws enumerate
//! every mix of two quiescent published states, and one half-length file over
//! the newer state, so they need no scheduling or timing. Every such copy must
//! refuse or return exactly the bytes a content identity names.
//!
//! They do not cover a copy that holds a writer's in-flight stage, a file at
//! another length than half, or a tear over the older state.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use keep::{
    CatalogRestartError, DurableOutcome, DurableReadError, DurableStore, DurableStoreError,
    FilesystemRetentionSnapshotError, ReaderAttemptLimit, RetentionClosureVerificationError,
    RetentionViewError, SegmentReadError, StoreExpectation,
};

use super::durable_fixture;
use super::durable_sandbox::TestDirectory;
use super::populated_store;
use super::store_copy::copy_store;
use super::writer_fixture::{
    Predecessor, TestResult, first_blob, open_publisher, publish, retain, second_blob,
};

/// Every changed file between the two states costs a doubling of the mixes.
const MAXIMUM_CHANGED_FILES: usize = 12;

type Files = BTreeMap<PathBuf, Vec<u8>>;

/// How many reads refused and how many returned bytes, over torn copies only.
#[derive(Default)]
struct Outcomes {
    refused: usize,
    /// Refusals at an absent or malformed artifact. A blob the selected state
    /// never held is refused by a whole state too, so it does not count here.
    damaged: usize,
    read: usize,
}

fn files(root: &Path) -> TestResult<Files> {
    let mut found = Files::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory)? {
            let path = entry?.path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let bytes = fs::read(&path)?;
                found.insert(path.strip_prefix(root)?.to_path_buf(), bytes);
            }
        }
    }
    Ok(found)
}

/// Builds a store, records it, and publishes a successor: the two torn-copy ends.
fn two_states(name: &str) -> TestResult<(TestDirectory, Files, Vec<PathBuf>, Files)> {
    let store = durable_fixture::build(name, &[&first_blob()])?;
    let before_copy = TestDirectory::create(&format!("{name}-before"))?;
    copy_store(store.path(), before_copy.path())?;
    let second = second_blob();
    let mut writer = open_publisher(store.path())?;
    let _receipt = publish(&mut writer, store.path(), &second, Predecessor::Kept)??;
    drop(writer);
    retain(store.path(), b"second", &second)?;
    let after = files(store.path())?;
    let before = files(before_copy.path())?;
    let changed: Vec<PathBuf> = after
        .iter()
        .filter(|(path, bytes)| before.get(*path) != Some(*bytes))
        .map(|(path, _)| path.clone())
        .collect();
    assert!(!changed.is_empty(), "the publication changed no file");
    assert!(
        changed.len() <= MAXIMUM_CHANGED_FILES,
        "{} changed files would make the enumeration too large",
        changed.len()
    );
    Ok((before_copy, before, changed, after))
}

/// Overlays `after`'s version of each selected file onto a copy of the base.
fn torn(name: &str, base: &Path, overlay: &[(&Path, &[u8])]) -> TestResult<TestDirectory> {
    let copy = TestDirectory::create(name)?;
    copy_store(base, copy.path())?;
    for (path, bytes) in overlay {
        let target = copy.path().join(path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(target, bytes)?;
    }
    Ok(copy)
}

/// Whether an I/O cause is evidence about the copy: an absent or malformed artifact.
fn evidenced(source: &io::Error) -> bool {
    matches!(
        source.kind(),
        io::ErrorKind::NotFound | io::ErrorKind::InvalidData
    )
}

/// Whether a segment refused for its own bytes and not for a policy or resource limit.
const fn malformed(source: &SegmentReadError) -> bool {
    matches!(
        source,
        SegmentReadError::WrongLength { .. }
            | SegmentReadError::Header { .. }
            | SegmentReadError::Seal { .. }
            | SegmentReadError::RecordHeaderTruncated { .. }
            | SegmentReadError::RecordHeader { .. }
            | SegmentReadError::RecordTruncated { .. }
            | SegmentReadError::RecordDecode { .. }
    )
}

/// Whether `refusal` is evidence that the copy is incomplete or damaged.
///
/// Anything else, such as a permission, resource, or locator failure, is an
/// operational failure of the test and must not count as a refusal.
fn lawful(refusal: &DurableOutcome) -> bool {
    use FilesystemRetentionSnapshotError as Snapshot;
    match refusal {
        DurableOutcome::Read(DurableReadError::BlobMissing { .. }) => true,
        DurableOutcome::Store(DurableStoreError::Closure { source, .. }) => matches!(
            **source,
            RetentionClosureVerificationError::MissingMember { .. }
        ),
        DurableOutcome::Store(DurableStoreError::Snapshot(source)) => match &**source {
            Snapshot::Admission { source }
            | Snapshot::Root { source }
            | Snapshot::View {
                source: RetentionViewError::Io { source, .. },
            } => evidenced(source),
            Snapshot::Catalog { source } => match source {
                CatalogRestartError::Io { source, .. }
                | CatalogRestartError::SegmentIo { source, .. } => evidenced(source),
                CatalogRestartError::Length { .. } => true,
                CatalogRestartError::Segment { source, .. } => malformed(source),
                _ => false,
            },
            _ => false,
        },
        _ => false,
    }
}

/// Requires `content` to reconstruct exactly, or to refuse lawfully with a verified prefix.
fn exact_or_refused(root: &Path, content: &[u8], outcomes: &mut Outcomes) -> TestResult<()> {
    let target = populated_store::identify(content)?.target;
    let store = DurableStore::open(
        root,
        populated_store::policy()?,
        ReaderAttemptLimit::DEFAULT,
        StoreExpectation::NoExpectation,
    )?;
    let mut output = Vec::new();
    match store.reconstruct(target, &mut output) {
        Ok(_receipt) => {
            assert_eq!(output, content, "a torn copy returned other bytes");
            outcomes.read = outcomes.read.saturating_add(1);
        }
        Err(refusal) => {
            assert!(
                lawful(&refusal),
                "a torn copy failed for another cause than its own state: {refusal:?}"
            );
            assert!(
                content.starts_with(&output),
                "a refused read emitted bytes its identity does not name"
            );
            outcomes.refused = outcomes.refused.saturating_add(1);
            if !matches!(
                refusal,
                DurableOutcome::Read(DurableReadError::BlobMissing { .. })
            ) {
                outcomes.damaged = outcomes.damaged.saturating_add(1);
            }
        }
    }
    Ok(())
}

fn both_blobs(root: &Path, outcomes: &mut Outcomes) -> TestResult<()> {
    exact_or_refused(root, &first_blob(), outcomes)?;
    exact_or_refused(root, &second_blob(), outcomes)
}

#[test]
fn every_mix_of_two_quiescent_published_states_refuses_or_returns_exact_bytes() -> TestResult<()> {
    let (base, _before, changed, after) = two_states("torn-mix")?;
    let newer = (1_u32 << changed.len()).saturating_sub(1);
    let mut torn_outcomes = Outcomes::default();
    for mask in 0_u32..=newer {
        let mut overlay: Vec<(&Path, &[u8])> = Vec::new();
        for (bit, path) in changed.iter().enumerate() {
            if mask & (1_u32 << bit) != 0 {
                let bytes = after.get(path).ok_or("changed file left the state")?;
                overlay.push((path, bytes));
            }
        }
        let copy = torn("torn-mix-copy", base.path(), &overlay)?;
        // The two ends are whole states. They must hold the law, but only a
        // real mix may satisfy the guards below.
        let mut whole_state = Outcomes::default();
        let outcomes = if mask == 0 || mask == newer {
            &mut whole_state
        } else {
            &mut torn_outcomes
        };
        both_blobs(copy.path(), outcomes)?;
    }
    assert!(torn_outcomes.read > 0, "no torn mix read anything");
    assert!(torn_outcomes.refused > 0, "no torn mix refused anything");
    assert!(
        torn_outcomes.damaged > 0,
        "no torn mix refused at an absent or malformed artifact"
    );
    Ok(())
}

#[test]
fn a_half_length_copy_of_any_one_file_over_the_newer_state_refuses_or_returns_exact_bytes()
-> TestResult<()> {
    let (base, before, changed, after) = two_states("torn-partial")?;
    let mut outcomes = Outcomes::default();
    let whole: Vec<(&Path, &[u8])> = after
        .iter()
        .filter(|(path, _)| changed.contains(path))
        .map(|(path, bytes)| (path.as_path(), bytes.as_slice()))
        .collect();
    for path in before.keys().chain(changed.iter()) {
        let source = after.get(path).or_else(|| before.get(path));
        let bytes = source.ok_or("the file left both states")?;
        if bytes.is_empty() {
            // Half of an empty lock file is the whole file: not a tear.
            continue;
        }
        let half = bytes
            .get(..bytes.len() >> 1)
            .ok_or("half is out of range")?;
        let mut overlay = whole.clone();
        overlay.retain(|(candidate, _)| *candidate != path.as_path());
        overlay.push((path.as_path(), half));
        let copy = torn("torn-partial-copy", base.path(), &overlay)?;
        both_blobs(copy.path(), &mut outcomes)?;
    }
    assert!(
        outcomes.damaged > 0,
        "no partial copy refused at a malformed artifact"
    );
    Ok(())
}
