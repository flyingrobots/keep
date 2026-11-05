//! This module owns the ADR-0010 law for a copy torn by a concurrent writer.
//!
//! A copy taken while a writer publishes can hold any mix of the states before
//! and after that publication, and a partly copied file. These laws enumerate
//! those mixes directly, so they need no scheduling or timing. Every mix must
//! refuse or return exactly the bytes a content identity names.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use keep::{DurableStore, ReaderAttemptLimit, StoreExpectation};

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

/// How many mixes refused and how many read, so the laws cannot pass vacuously.
#[derive(Default)]
struct Outcomes {
    refused: usize,
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

/// Requires `content` to reconstruct exactly, or to refuse with a verified prefix.
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
        Err(_refusal) => {
            assert!(
                content.starts_with(&output),
                "a refused read emitted bytes its identity does not name"
            );
            outcomes.refused = outcomes.refused.saturating_add(1);
        }
    }
    Ok(())
}

fn both_blobs(root: &Path, outcomes: &mut Outcomes) -> TestResult<()> {
    exact_or_refused(root, &first_blob(), outcomes)?;
    exact_or_refused(root, &second_blob(), outcomes)
}

#[test]
fn every_mix_of_two_published_states_refuses_or_returns_exact_bytes() -> TestResult<()> {
    let (base, _before, changed, after) = two_states("torn-mix")?;
    let mut outcomes = Outcomes::default();
    for mask in 0_u32..(1_u32 << changed.len()) {
        let mut overlay: Vec<(&Path, &[u8])> = Vec::new();
        for (bit, path) in changed.iter().enumerate() {
            if mask & (1_u32 << bit) != 0 {
                let bytes = after.get(path).ok_or("changed file left the state")?;
                overlay.push((path, bytes));
            }
        }
        let copy = torn("torn-mix-copy", base.path(), &overlay)?;
        both_blobs(copy.path(), &mut outcomes)?;
    }
    assert!(outcomes.read > 0, "no torn mix read anything");
    assert!(outcomes.refused > 0, "no torn mix refused anything");
    Ok(())
}

#[test]
fn every_partly_copied_file_refuses_or_returns_exact_bytes() -> TestResult<()> {
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
        let half = bytes
            .get(..bytes.len() >> 1)
            .ok_or("half is out of range")?;
        let mut overlay = whole.clone();
        overlay.retain(|(candidate, _)| *candidate != path.as_path());
        overlay.push((path.as_path(), half));
        let copy = torn("torn-partial-copy", base.path(), &overlay)?;
        both_blobs(copy.path(), &mut outcomes)?;
    }
    assert!(outcomes.refused > 0, "no partial copy refused anything");
    Ok(())
}
