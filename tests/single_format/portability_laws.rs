//! This module owns the ADR-0010 portability laws: a store is not bound to its directory.
//!
//! Each law copies a quiescent store byte for byte into a new directory, so
//! every inode differs, including the root's. Admission must treat the copy
//! as the same store, refuse a damaged copy exactly as it refuses the
//! damaged original, and keep every live check.

use std::fs;
use std::io;
use std::path::Path;

use keep::{
    FilesystemCatalogPublisher, FilesystemStoreAdmission, FilesystemStoreAdmissionError,
    RetainedCatalogPublicationError, RetentionCurrentStateRefusal, StoreRecordRefusal,
    WriterLockAcquireError,
};

use super::durable_fixture;
use super::durable_sandbox::TestDirectory;
use super::populated_store;
use super::store_copy::copy_store;
use super::writer_fixture::{
    Predecessor, TestResult, assert_reads, first_blob, open_publisher, pool_segments, publish,
    retain, second_blob,
};

/// A quiescent copy of a retained store under a new directory.
fn copied(name: &str, original: &Path) -> TestResult<TestDirectory> {
    let copy = TestDirectory::create(name)?;
    copy_store(original, copy.path())?;
    Ok(copy)
}

#[test]
fn a_valid_quiescent_copy_opens_and_reads_the_same_content() -> TestResult<()> {
    let first = first_blob();
    let original = durable_fixture::build("portable-original", &[&first])?;
    let copy = copied("portable-copy", original.path())?;

    drop(FilesystemStoreAdmission::reopen(copy.path())?);

    assert_reads(copy.path(), &first)
}

/// The same damage refuses with the same typed cause in the copy and the original.
#[test]
fn a_corrupt_copy_refuses_exactly_as_the_corrupt_original_would() -> TestResult<()> {
    let original = durable_fixture::build("portable-corrupt-original", &[&first_blob()])?;
    let copy = copied("portable-corrupt-copy", original.path())?;
    for root in [original.path(), copy.path()] {
        let marker = root.join("FORMAT");
        let mut bytes = fs::read(&marker)?;
        *bytes.get_mut(40).ok_or("marker shorter than 41 bytes")? ^= 1;
        fs::write(&marker, &bytes)?;
    }

    let original_refusal = marker_refusal(original.path())?;
    let copy_refusal = marker_refusal(copy.path())?;

    assert_eq!(copy_refusal, original_refusal);
    Ok(())
}

#[test]
fn an_incomplete_copy_refuses_exactly_as_the_incomplete_original_would() -> TestResult<()> {
    let original = durable_fixture::build("portable-incomplete-original", &[&first_blob()])?;
    let copy = copied("portable-incomplete-copy", original.path())?;
    for root in [original.path(), copy.path()] {
        fs::remove_dir(root.join("gc"))?;
    }

    let original_refusal = namespace_refusal_kind(original.path())?;
    let copy_refusal = namespace_refusal_kind(copy.path())?;

    assert_eq!(copy_refusal, original_refusal);
    assert_eq!(copy_refusal, io::ErrorKind::NotFound);
    Ok(())
}

/// Live authority still binds the copy's own lock file and directories: a
/// writer lock replaced while a writer holds the store refuses a second
/// writer, and a retention directory replaced after admission refuses
/// publication.
#[test]
fn a_replaced_lock_or_protocol_directory_during_an_operation_still_refuses() -> TestResult<()> {
    let original = durable_fixture::build("portable-replaced-original", &[&first_blob()])?;
    let copy = copied("portable-replaced-copy", original.path())?;
    let root = copy.path();
    let held = FilesystemStoreAdmission::reopen(root)?;
    fs::remove_file(root.join("writer.lock"))?;
    fs::write(root.join("writer.lock"), b"")?;

    let contended = FilesystemStoreAdmission::reopen(root);

    assert!(matches!(
        contended,
        Err(FilesystemStoreAdmissionError::WriterLock {
            source: WriterLockAcquireError::Busy
        })
    ));
    // The displaced directory leaves the root, so every path-based reader still
    // admits it; only the publisher's pinned capability can notice.
    let displaced = TestDirectory::create("portable-replaced-displaced")?;
    fs::rename(root.join("retention"), displaced.path().join("retention"))?;
    fs::create_dir_all(root.join("retention/roots"))?;
    fs::create_dir(root.join("retention/manifests"))?;
    let mut publisher = FilesystemCatalogPublisher::open(held, populated_store::policy()?)?;
    let outcome = publish(&mut publisher, root, &second_blob(), Predecessor::Kept)?;
    let Err(RetainedCatalogPublicationError::RetentionStorage { source, .. }) = outcome else {
        return Err(format!("publication used a replaced retention directory: {outcome:?}").into());
    };
    assert!(matches!(
        source
            .get_ref()
            .and_then(|cause| cause.downcast_ref::<RetentionCurrentStateRefusal>()),
        Some(RetentionCurrentStateRefusal::ProtocolDirectoryReplaced)
    ));
    Ok(())
}

/// A copy and its original are two stores: both take writer authority at
/// once, each publishes its own successor, and the content they still share
/// keeps its exact identity and bytes in both.
#[test]
fn a_copy_and_its_original_diverge_independently_and_keep_shared_identity() -> TestResult<()> {
    let first = first_blob();
    let original = durable_fixture::build("portable-diverge-original", &[&first])?;
    let copy = copied("portable-diverge-copy", original.path())?;
    let second = second_blob();
    let other = b"keep portability: content only the copy publishes. ".repeat(2_000);
    let shared = pool_segments(original.path())?;
    let mut original_writer = open_publisher(original.path())?;
    let mut copy_writer = open_publisher(copy.path())?;

    let _original = publish(
        &mut original_writer,
        original.path(),
        &second,
        Predecessor::Kept,
    )??;
    let _copy = publish(&mut copy_writer, copy.path(), &other, Predecessor::Kept)??;
    drop((original_writer, copy_writer));
    retain(original.path(), b"original-only", &second)?;
    retain(copy.path(), b"copy-only", &other)?;

    for root in [original.path(), copy.path()] {
        assert_reads(root, &first)?;
    }
    assert_reads(original.path(), &second)?;
    assert_reads(copy.path(), &other)?;
    for root in [original.path(), copy.path()] {
        let pool = pool_segments(root)?;
        assert!(
            shared.iter().all(|segment| pool.contains(segment)),
            "unchanged content must keep its exact segment bytes in both stores"
        );
    }
    assert_ne!(
        fs::read(original.path().join("HEAD"))?,
        fs::read(copy.path().join("HEAD"))?,
        "independent publications must produce different catalogs"
    );
    Ok(())
}

fn marker_refusal(root: &Path) -> TestResult<String> {
    let Err(FilesystemStoreAdmissionError::FormatMarker { source }) =
        FilesystemStoreAdmission::reopen(root)
    else {
        return Err("a corrupt marker was not refused at the marker boundary".into());
    };
    let refusal = source
        .get_ref()
        .and_then(|cause| cause.downcast_ref::<StoreRecordRefusal>())
        .ok_or("the marker refusal lost its typed cause")?;
    Ok(format!("{refusal:?}"))
}

fn namespace_refusal_kind(root: &Path) -> TestResult<io::ErrorKind> {
    match FilesystemStoreAdmission::reopen(root) {
        Err(FilesystemStoreAdmissionError::Namespace { source }) => Ok(source.kind()),
        Err(other) => Err(format!("unexpected incomplete-store refusal: {other:?}").into()),
        Ok(_) => Err("an incomplete store was admitted".into()),
    }
}
