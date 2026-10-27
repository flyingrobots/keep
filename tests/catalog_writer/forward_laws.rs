//! This module owns forward ingestion laws.

use keep::{CatalogPublicationOutcome, DurableStore, ReaderAttemptLimit};

use super::durable_fixture;
use super::writer_fixture::{
    Predecessor, TestResult, assert_reads, first_blob, open_publisher, publish, retain,
    second_blob, witness,
};

#[test]
fn a_successor_that_preserves_retained_content_publishes_and_reads_back() -> TestResult<()> {
    let first = first_blob();
    let second = second_blob();
    let sandbox = durable_fixture::build("v2-writer-forward", &[&first])?;
    let root = sandbox.path();
    let mut publisher = open_publisher(root)?;
    let receipt = publish(&mut publisher, root, &second, Predecessor::Kept)??;
    drop(publisher);

    assert_eq!(receipt.generation().get(), 2);
    assert_eq!(receipt.outcome(), CatalogPublicationOutcome::Published);
    let preserved = receipt
        .preserved_manifest()
        .ok_or("receipt must bind the preserved retention manifest")?;
    assert_eq!(preserved.generation().get(), 1);
    assert_eq!(preserved.root_count(), 1);
    let staging = witness(&root.join("staging"))?;
    assert!(
        staging.is_empty(),
        "publication left stage residue: {staging:?}"
    );

    // Every retained root still verifies against the successor catalog.
    assert_reads(root, &first)?;
    retain(root, b"second", &second)?;
    assert_reads(root, &first)?;
    assert_reads(root, &second)
}

#[test]
fn publication_runs_beside_a_pinned_reader_view() -> TestResult<()> {
    let first = first_blob();
    let second = second_blob();
    let sandbox = durable_fixture::build("v2-writer-reader", &[&first])?;
    let root = sandbox.path();
    let store = DurableStore::open(
        root,
        durable_fixture::policy()?,
        ReaderAttemptLimit::DEFAULT,
    )?;
    let view = store.snapshot()?;

    let mut publisher = open_publisher(root)?;
    let _receipt = publish(&mut publisher, root, &second, Predecessor::Kept)??;
    drop(publisher);

    let target = durable_fixture::identify(&first)?.target;
    let mut output = Vec::new();
    let _receipt = view.reconstruct(target, &mut output)?;
    assert_eq!(output, first, "the pinned view must keep its generation");
    drop(view);
    assert_reads(root, &first)
}
