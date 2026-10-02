//! Durable Worldline reads through public production admission and publication.
//!
//! Size: medium; owned admitted ext4 storage, no ambient network or sleeps.
//! Oracle: Worldline's independently frozen identities and original input bytes.
//! Delete when durable reads are removed or stronger corpus witnesses subsume these laws.

use std::error::Error;

use keep::{ByteLength, ByteOffset, ByteRange, DurableStore, ReaderAttemptLimit};

use super::durable_fixture::{build, policy};
use super::identity_corpus::{IdentityCase, identity_cases};

#[test]
fn reopened_durable_views_reconstruct_every_worldline_identity() -> Result<(), Box<dyn Error>> {
    let cases = identity_cases()?;
    let bytes = cases
        .iter()
        .map(IdentityCase::bytes)
        .collect::<Result<Vec<_>, _>>()?;
    let sources = bytes.iter().map(Vec::as_slice).collect::<Vec<_>>();
    let sandbox = build("durable-worldline-reopen", &sources)?;
    for (case, expected) in cases.iter().zip(&bytes) {
        let store = DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT);
        let target = case.expected_id()?;
        let mut output = Vec::new();
        let receipt = store.reconstruct(target, &mut output)?;
        assert_eq!(
            &output, expected,
            "reopened durable bytes for {}",
            case.name
        );
        assert_eq!(
            receipt.receipt().target(),
            target,
            "frozen identity for {}",
            case.name
        );
        assert_eq!(
            receipt.receipt().bytes_written().get(),
            u64::try_from(expected.len())?
        );
        assert!(
            store.contains_blob(target)?,
            "published Worldline identity must remain discoverable"
        );
    }
    sandbox.remove()?;
    Ok(())
}

#[test]
fn durable_worldline_ranges_match_source_slices_after_reopen() -> Result<(), Box<dyn Error>> {
    let cases = identity_cases()?;
    let bytes = cases
        .iter()
        .map(IdentityCase::bytes)
        .collect::<Result<Vec<_>, _>>()?;
    let sources = bytes.iter().map(Vec::as_slice).collect::<Vec<_>>();
    let sandbox = build("durable-worldline-ranges", &sources)?;
    for (case, expected) in cases.iter().zip(&bytes) {
        let store = DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT);
        let snapshot = store.snapshot()?;
        let target = case.expected_id()?;
        let length = u64::try_from(expected.len())?;
        for (start, end) in [
            (0, 0),
            (0, length),
            (length, length),
            (
                length.checked_div(3).ok_or("zero divisor")?,
                length
                    .checked_sub(length.checked_div(3).ok_or("zero divisor")?)
                    .ok_or("inverted interval")?,
            ),
        ] {
            let requested = ByteRange::new(
                ByteOffset::new(start),
                ByteLength::new(end.checked_sub(start).ok_or("inverted interval")?),
            )?;
            let mut output = Vec::new();
            let receipt = snapshot.read_range(target, requested, &mut output)?;
            assert_eq!(
                Some(output.as_slice()),
                expected.get(usize::try_from(start)?..usize::try_from(end)?),
                "range {requested:?} for {}",
                case.name
            );
            assert_eq!(receipt.receipt().target(), target);
            assert_eq!(receipt.receipt().requested(), requested);
            assert_eq!(receipt.view(), snapshot.view());
        }
    }
    sandbox.remove()?;
    Ok(())
}
