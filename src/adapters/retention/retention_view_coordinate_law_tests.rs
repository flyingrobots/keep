//! This module owns semantic reader retries for complete retention coordinates.

use std::collections::VecDeque;
use std::error::Error;
use std::io;

use super::{RetentionViewCoordinates, RetentionViewSource, collect_retention_view};
use crate::adapters::retention::ReaderAttemptLimit;
use crate::{
    CatalogDigest, CatalogGeneration, CatalogLength, LivenessGeneration, RetentionHead,
    RetentionManifestDigest, RetentionManifestLength,
};

struct Views {
    heads: VecDeque<RetentionViewCoordinates>,
    values: VecDeque<&'static str>,
}

impl RetentionViewSource for Views {
    type View = &'static str;

    fn coordinates(&mut self) -> io::Result<RetentionViewCoordinates> {
        self.heads
            .pop_front()
            .ok_or_else(|| io::Error::other("coordinate script exhausted"))
    }

    fn load(&mut self) -> io::Result<Self::View> {
        self.values
            .pop_front()
            .ok_or_else(|| io::Error::other("view script exhausted"))
    }
}

fn head(
    length: RetentionManifestLength,
    predecessor: [u8; 32],
) -> Result<RetentionHead, Box<dyn Error>> {
    Ok(RetentionHead::new(
        LivenessGeneration::new(2)?,
        length,
        RetentionManifestDigest::from_hash([255; 32]),
        Some(RetentionManifestDigest::from_hash(predecessor)),
    )?)
}

fn retry(before: RetentionHead, after: RetentionHead) -> Result<(), Box<dyn Error>> {
    let catalog = Some((
        CatalogGeneration::new(1)?,
        CatalogLength::MINIMUM,
        CatalogDigest::from_validated([0; 32]),
    ));
    let before = RetentionViewCoordinates {
        catalog,
        retention: Some(before),
    };
    let after = RetentionViewCoordinates {
        catalog,
        retention: Some(after),
    };
    let mut source = Views {
        heads: [before, after, after, after].into(),
        values: ["superseded", "stable"].into(),
    };
    let view = collect_retention_view(&mut source, ReaderAttemptLimit::DEFAULT)?;
    assert_eq!(
        view, "stable",
        "a changed retention coordinate must discard the superseded view: {before:?} -> {after:?}"
    );
    Ok(())
}

// Size: small. Oracle: changing a selected manifest length invalidates that attempt.
// Domain: every admitted length above the minimum; no snapshots or harness-count oracle.
// Delete when full-head collection is removed or a stronger cheaper law subsumes this domain.
#[test]
fn every_changed_manifest_length_discards_the_superseded_view() -> Result<(), Box<dyn Error>> {
    let before = head(RetentionManifestLength::MINIMUM, [0; 32])?;
    for length in (RetentionManifestLength::MINIMUM.get()..=RetentionManifestLength::MAXIMUM.get())
        .step_by(72)
        .skip(1)
    {
        retry(
            before,
            head(RetentionManifestLength::new(length)?, [0; 32])?,
        )?;
    }
    Ok(())
}

// Size: small. Oracle: changing a predecessor invalidates that collection attempt.
// Domain: every nonzero first byte, with other predecessor bytes held at zero.
// Delete when full-head collection is removed or a stronger cheaper law subsumes this domain.
#[test]
fn changed_predecessor_bytes_discard_the_superseded_view() -> Result<(), Box<dyn Error>> {
    let before = head(RetentionManifestLength::MINIMUM, [0; 32])?;
    for byte in 1..=u8::MAX {
        let mut predecessor = [0; 32];
        *predecessor.first_mut().ok_or("predecessor absent")? = byte;
        retry(before, head(RetentionManifestLength::MINIMUM, predecessor)?)?;
    }
    Ok(())
}
