//! The generic port laws run against a pinned durable snapshot.

use std::error::Error;

use super::DurableStore;
use super::test_fixture::{durable_store, long_content};
use crate::BlobHasher;
use crate::adapters::ReaderAttemptLimit;
use crate::adapters::retention::filesystem_retention_test_fixture::catalog_policy;
use crate::store::port_laws::{absence_refuses, ranges_exactly, reconstructs_exactly};

#[test]
fn durable_backend_satisfies_the_read_laws_through_the_port() -> Result<(), Box<dyn Error>> {
    let long = long_content();
    let contents: [&[u8]; 3] = [b"durable through the port", &long, b"unanchored"];
    let (sandbox, published) = durable_store("durable-port-laws", &contents)?;
    let snapshot = DurableStore::open(
        sandbox.path(),
        catalog_policy()?,
        ReaderAttemptLimit::DEFAULT,
    )?
    .snapshot()?;
    for (bytes, entry) in contents.iter().zip(&published).take(2) {
        reconstructs_exactly(&snapshot, entry.target, entry.layout, bytes)?;
    }
    let anchored = published.get(1).ok_or("second published blob")?;
    ranges_exactly(
        &snapshot,
        anchored.target,
        &long,
        &[(0, 1), (65_000, 5_000), (400_000, 42 * 1024)],
    )?;
    absence_refuses(&snapshot, BlobHasher::new().finish())?;
    Ok(())
}
