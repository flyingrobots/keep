//! Durable failures render each boundary without repeating the preserved cause.
//!
//! Size: medium. Oracle: each boundary describes itself; `Error::source` retains
//! the original typed cause for reporters to render separately.
//! Delete when durable read APIs disappear or stronger diagnostic evidence subsumes these laws.

use super::durable_fixture::{build, identify, policy};
use crate::support::ZeroWriter;
use keep::{ByteLength, ByteOffset, ByteRange, DurableStore, ReaderAttemptLimit};
use std::error::Error;

#[test]
fn durable_whole_failure_renders_each_boundary_once() -> Result<(), Box<dyn Error>> {
    let bytes = b"diagnostic whole output";
    let sandbox = build("durable-whole-diagnostic", &[bytes])?;
    let store = DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT)?;
    let failure = store
        .reconstruct(identify(bytes)?.target, &mut ZeroWriter)
        .err()
        .ok_or("zero writer succeeded")?;
    let read = failure.source().ok_or("read cause absent")?;
    let core = read.source().ok_or("core cause absent")?;
    assert!(
        !failure.to_string().contains(&read.to_string()),
        "store boundary repeats read cause: {failure}"
    );
    assert!(
        !read.to_string().contains(&core.to_string()),
        "read boundary repeats core cause: {read}"
    );
    Ok(())
}

#[test]
fn durable_range_failure_renders_each_boundary_once() -> Result<(), Box<dyn Error>> {
    let bytes = b"diagnostic range output";
    let sandbox = build("durable-range-diagnostic", &[bytes])?;
    let store = DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT)?;
    let range = ByteRange::new(ByteOffset::new(0), ByteLength::new(1))?;
    let failure = store
        .read_range(identify(bytes)?.target, range, &mut ZeroWriter)
        .err()
        .ok_or("zero range writer succeeded")?;
    let read = failure.source().ok_or("read cause absent")?;
    let core = read.source().ok_or("core cause absent")?;
    assert!(
        !failure.to_string().contains(&read.to_string()),
        "store boundary repeats range cause: {failure}"
    );
    assert!(
        !read.to_string().contains(&core.to_string()),
        "range boundary repeats core cause: {read}"
    );
    Ok(())
}

#[test]
fn durable_admission_failure_renders_its_boundary_once() -> Result<(), Box<dyn Error>> {
    let bytes = b"diagnostic admission";
    let sandbox = build("durable-admission-diagnostic", &[bytes])?;
    let store = DurableStore::open(
        &sandbox.path().join("absent"),
        policy()?,
        ReaderAttemptLimit::DEFAULT,
    )?;
    let failure = store
        .reconstruct(identify(bytes)?.target, &mut Vec::new())
        .err()
        .ok_or("absent store admitted")?;
    let cause = failure.source().ok_or("admission cause absent")?;
    assert!(
        !failure.to_string().contains(&cause.to_string()),
        "store boundary repeats admission cause: {failure}"
    );
    Ok(())
}
