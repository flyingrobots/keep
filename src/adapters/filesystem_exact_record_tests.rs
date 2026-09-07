//! Exact-record primitive laws: bounded reads, identity reverification, absence, no replacement.

use std::error::Error;
use std::fs;

use cap_std::fs::Dir;

use super::{EntryIdentity, ExactRecordError, ExactRecordRefusal, read_exact_optional};
use crate::adapters::filesystem_test_sandbox::TestDirectory;

fn open(sandbox: &TestDirectory) -> Result<Dir, Box<dyn Error>> {
    Ok(Dir::open_ambient_dir(
        sandbox.path(),
        cap_std::ambient_authority(),
    )?)
}

fn refusal(error: ExactRecordError) -> Result<ExactRecordRefusal, Box<dyn Error>> {
    match error {
        ExactRecordError::Refused(refusal) => Ok(refusal),
        ExactRecordError::Io(source) => Err(source.into()),
    }
}

#[test]
fn absent_record_reads_as_none_and_exact_record_reads_its_bytes() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("exact-record-optional")?;
    let directory = open(&sandbox)?;
    fs::write(sandbox.path().join("record"), b"exact")?;

    assert!(read_exact_optional(&directory, "absent", 5)?.is_none());
    assert_eq!(
        read_exact_optional(&directory, "record", 5)?.as_deref(),
        Some(b"exact".as_slice())
    );
    drop(directory);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn wrong_length_or_kind_refuses_before_reading() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("exact-record-kind-or-length")?;
    let directory = open(&sandbox)?;
    fs::write(sandbox.path().join("short"), b"abc")?;
    fs::create_dir(sandbox.path().join("directory"))?;

    let short = read_exact_optional(&directory, "short", 5)
        .err()
        .ok_or("short record admitted")?;
    let kind = read_exact_optional(&directory, "directory", 5)
        .err()
        .ok_or("directory admitted as a record")?;

    assert_eq!(refusal(short)?, ExactRecordRefusal::KindOrLength);
    assert_eq!(refusal(kind)?, ExactRecordRefusal::KindOrLength);
    drop(directory);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn verify_named_refuses_different_bytes_and_a_byte_equal_substitute() -> Result<(), Box<dyn Error>>
{
    let sandbox = TestDirectory::create("exact-record-verify")?;
    let directory = open(&sandbox)?;
    let path = sandbox.path().join("record");
    fs::write(&path, b"exact")?;
    let identity = EntryIdentity::of_file(&directory.open("record")?)?;

    super::verify_named(&directory, "record", b"exact", identity)?;
    let bytes = super::verify_named(&directory, "record", b"other", identity)
        .err()
        .ok_or("different bytes admitted")?;
    fs::remove_file(&path)?;
    fs::write(&path, b"exact")?;
    let substitute = super::verify_named(&directory, "record", b"exact", identity)
        .err()
        .ok_or("byte-equal substitute admitted")?;

    assert_eq!(refusal(bytes)?, ExactRecordRefusal::Bytes);
    assert_eq!(
        refusal(substitute)?,
        ExactRecordRefusal::KindLengthOrIdentity
    );
    drop(directory);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn require_absent_refuses_a_visible_entry_and_links_never_replace() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("exact-record-absent-link")?;
    let directory = open(&sandbox)?;
    fs::write(sandbox.path().join("stage"), b"stage")?;
    fs::write(sandbox.path().join("target"), b"existing")?;

    let visible = super::require_absent(&directory, "stage")
        .err()
        .ok_or("visible entry admitted as absent")?;
    super::link_without_replacement(&directory, "stage", &directory, "target")?;
    super::link_without_replacement(&directory, "stage", &directory, "linked")?;

    assert_eq!(refusal(visible)?, ExactRecordRefusal::RemainedVisible);
    assert_eq!(fs::read(sandbox.path().join("target"))?, b"existing");
    assert_eq!(fs::read(sandbox.path().join("linked"))?, b"stage");
    super::require_absent(&directory, "absent")?;
    drop(directory);
    sandbox.remove()?;
    Ok(())
}
