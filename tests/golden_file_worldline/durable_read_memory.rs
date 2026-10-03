//! Incremental read allocation after explicit durable snapshot materialization.
//!
//! Size: medium. Oracle: reading the 1 MiB Worldline source must not allocate a
//! second whole-blob buffer; the already-admitted snapshot is outside this scope.
//! Delete when the no-additional-whole-blob-buffer contract changes or stronger
//! generated allocation evidence subsumes this concrete witness.

use std::error::Error;
use std::io;

use super::durable_fixture::{build, policy};
use super::identity_corpus::find_case;
use keep::{DurableStore, ReaderAttemptLimit};

#[test]
fn durable_reconstruction_does_not_allocate_an_additional_whole_blob() -> Result<(), Box<dyn Error>>
{
    let case = find_case("large-ramp")?;
    let bytes = case.bytes()?;
    let sandbox = build("durable-read-allocation", &[&bytes])?;
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT)?.snapshot()?;
    let target = case.expected_id()?;
    let mut output = io::sink();
    let mut result = None;
    let allocation = allocation_counter::measure(|| {
        result = Some(snapshot.reconstruct(target, &mut output));
    });
    let receipt = result.ok_or("read was not executed")??;
    let length = u64::try_from(bytes.len())?;
    assert_eq!(receipt.receipt().bytes_written().get(), length);
    assert!(
        allocation.bytes_max < length,
        "incremental read allocation {} must be below the {length}-byte blob; snapshot materialization is separately bounded by policy",
        allocation.bytes_max
    );
    Ok(())
}
