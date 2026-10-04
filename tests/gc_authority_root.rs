//! Medium filesystem laws: mutation authority and the observation locator name one store.
//! Oracle: admitted device/inode identity and unchanged bytes in both complete stores.
//! Delete only if both authority APIs disappear or stronger public laws subsume these cases.

#![cfg(target_os = "linux")]

#[allow(
    dead_code,
    reason = "the shared publication fixture also serves other runtime laws"
)]
#[path = "golden_file_worldline/durable_fixture.rs"]
#[expect(
    clippy::redundant_pub_crate,
    reason = "shared fixture retains sibling-only visibility at both test-module depths"
)]
mod durable_fixture;
#[allow(
    dead_code,
    reason = "sandbox removal is also available to other runtime laws"
)]
#[path = "segment_filesystem_stage/sandbox.rs"]
#[expect(
    clippy::redundant_pub_crate,
    reason = "shared sandbox retains sibling-only visibility at both test-module depths"
)]
mod durable_sandbox;

use keep::{
    FilesystemCompactionAuthority, FilesystemCompactionError, FilesystemGcAuthority,
    FilesystemGcError, FilesystemPlatformAdmissionError, FilesystemVersionTwoAdmission,
    StoreRootIdentityCoordinate,
};
use std::os::unix::fs::MetadataExt;
use std::{
    collections::BTreeMap,
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};

#[test]
fn gc_refuses_an_observation_locator_for_another_store() -> Result<(), Box<dyn Error>> {
    let admitted = durable_fixture::build("gc-root-admitted", &[b"retained in A"])?;
    let unrelated = durable_fixture::build("gc-root-unrelated", &[b"retained in B"])?;
    let before = (witness(admitted.path())?, witness(unrelated.path())?);
    let authority = FilesystemVersionTwoAdmission::reopen(admitted.path())?;
    let result =
        FilesystemGcAuthority::open(authority, unrelated.path(), durable_fixture::policy()?);
    let Err(FilesystemGcError::Observe { source }) = result else {
        return Err("GC must refuse mixed authority and observation roots before effects".into());
    };
    require_identity_refusal(&source, admitted.path(), unrelated.path())?;
    assert_eq!(
        (witness(admitted.path())?, witness(unrelated.path())?),
        before,
        "mixed-root refusal preserves every record in both stores"
    );
    Ok(())
}

#[test]
fn compaction_refuses_an_observation_locator_for_another_store() -> Result<(), Box<dyn Error>> {
    let admitted = durable_fixture::build("compaction-root-admitted", &[b"retained in A"])?;
    let unrelated = durable_fixture::build("compaction-root-unrelated", &[b"retained in B"])?;
    let before = (witness(admitted.path())?, witness(unrelated.path())?);
    let authority = FilesystemVersionTwoAdmission::reopen(admitted.path())?;
    let result = FilesystemCompactionAuthority::open(
        authority,
        unrelated.path(),
        durable_fixture::policy()?,
    );
    let Err(FilesystemCompactionError::Observe { source }) = result else {
        return Err(
            "compaction must refuse mixed authority and observation roots before effects".into(),
        );
    };
    require_identity_refusal(&source, admitted.path(), unrelated.path())?;
    assert_eq!(
        (witness(admitted.path())?, witness(unrelated.path())?),
        before,
        "mixed-root refusal preserves every record in both stores"
    );
    Ok(())
}

fn require_identity_refusal(
    error: &io::Error,
    admitted: &Path,
    unrelated: &Path,
) -> Result<(), Box<dyn Error>> {
    let Some(FilesystemPlatformAdmissionError::RootIdentityChanged {
        coordinate,
        expected,
        observed,
    }) = error
        .get_ref()
        .and_then(|source| source.downcast_ref::<FilesystemPlatformAdmissionError>())
    else {
        return Err("expected exact typed root identity mismatch".into());
    };
    assert_eq!(*coordinate, StoreRootIdentityCoordinate::File);
    assert_eq!(*expected, fs::metadata(admitted)?.ino());
    assert_eq!(*observed, fs::metadata(unrelated)?.ino());
    assert_ne!(expected, observed);
    Ok(())
}

fn witness(root: &Path) -> io::Result<BTreeMap<PathBuf, Vec<u8>>> {
    let mut files = BTreeMap::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            files.extend(witness(&entry.path())?);
        } else {
            files.insert(entry.path(), fs::read(entry.path())?);
        }
    }
    Ok(files)
}
