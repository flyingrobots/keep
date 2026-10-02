//! This module owns reader collection outcomes at the public source port.

use std::collections::VecDeque;
use std::error::Error;
use std::io;
use std::num::NonZeroU32;

use super::{
    ReaderAttemptLimit, RetentionViewCoordinates, RetentionViewError, RetentionViewSource,
    collect_retention_view,
};
use crate::{
    CatalogDigest, CatalogGeneration, CatalogLength, LivenessGeneration, RetentionHead,
    RetentionManifestDigest, RetentionManifestLength,
};

struct Scripted {
    coordinates: VecDeque<io::Result<RetentionViewCoordinates>>,
    views: VecDeque<io::Result<&'static str>>,
}

impl RetentionViewSource for Scripted {
    type View = &'static str;

    fn coordinates(&mut self) -> io::Result<RetentionViewCoordinates> {
        self.coordinates
            .pop_front()
            .ok_or_else(|| io::Error::other("coordinate script exhausted"))?
    }

    fn load(&mut self) -> io::Result<Self::View> {
        self.views
            .pop_front()
            .ok_or_else(|| io::Error::other("view script exhausted"))?
    }
}

fn published(
    generation: u64,
    digest: [u8; 32],
) -> Result<RetentionViewCoordinates, Box<dyn Error>> {
    Ok(RetentionViewCoordinates {
        catalog: Some((
            CatalogGeneration::new(generation)?,
            CatalogLength::MINIMUM,
            CatalogDigest::from_validated(digest),
        )),
        retention: None,
    })
}

const ABSENT: RetentionViewCoordinates = RetentionViewCoordinates {
    catalog: None,
    retention: None,
};

// Size: small. Oracle: stable coordinates return the loaded payload.
// Delete when the collector contract is retired or subsumed by stronger coverage.
#[test]
fn a_stable_store_returns_its_loaded_view() -> Result<(), Box<dyn Error>> {
    let stable = published(1, [0; 32])?;
    let mut source = Scripted {
        coordinates: [Ok(stable), Ok(stable)].into(),
        views: [Ok("stable payload")].into(),
    };
    assert_eq!(
        collect_retention_view(&mut source, ReaderAttemptLimit::DEFAULT)?,
        "stable payload"
    );
    Ok(())
}

fn require_retry(
    before: RetentionViewCoordinates,
    after: RetentionViewCoordinates,
) -> Result<(), Box<dyn Error>> {
    let mut source = Scripted {
        coordinates: [Ok(before), Ok(after), Ok(after), Ok(after)].into(),
        views: [Ok("superseded payload"), Ok("stable payload")].into(),
    };
    assert_eq!(
        collect_retention_view(&mut source, ReaderAttemptLimit::DEFAULT)?,
        "stable payload",
        "changed coordinates must discard the superseded payload: {before:?} -> {after:?}"
    );
    Ok(())
}

// Size: small. Oracle: a changed generation invalidates the loaded payload.
// Delete when the collector contract is retired or subsumed by stronger coverage.
#[test]
fn a_publication_between_reads_discards_the_superseded_view() -> Result<(), Box<dyn Error>> {
    require_retry(published(1, [0; 32])?, published(2, [0; 32])?)
}

// Size: small. Oracle: digest equality is required even at the same generation.
// Domain: every nonzero first byte; remaining digest bytes stay zero.
// Delete when a stronger digest-domain law subsumes this sweep.
#[test]
fn changed_catalog_digests_discard_the_superseded_view() -> Result<(), Box<dyn Error>> {
    for byte in 1..=u8::MAX {
        let mut digest = [0; 32];
        *digest.first_mut().ok_or("digest absent")? = byte;
        require_retry(published(1, [0; 32])?, published(1, digest)?)?;
    }
    Ok(())
}

// Size: small. Oracle: a selected retention digest change invalidates the loaded view.
// Domain: every nonzero first byte with other digest bytes fixed at zero.
// Delete when a stronger digest-domain law subsumes this sweep.
#[test]
fn changed_retention_digests_discard_the_superseded_view() -> Result<(), Box<dyn Error>> {
    let mut before = published(1, [0; 32])?;
    before.retention = Some(RetentionHead::new(
        LivenessGeneration::new(1)?,
        RetentionManifestLength::MINIMUM,
        RetentionManifestDigest::from_hash([0; 32]),
        None,
    )?);
    for byte in 1..=u8::MAX {
        let mut digest = [0; 32];
        *digest.first_mut().ok_or("digest absent")? = byte;
        let mut after = before;
        after.retention = Some(RetentionHead::new(
            LivenessGeneration::new(1)?,
            RetentionManifestLength::MINIMUM,
            RetentionManifestDigest::from_hash(digest),
            None,
        )?);
        require_retry(before, after)?;
    }
    Ok(())
}

// Size: small. Oracle: exhausted attempts refuse instead of returning a moving view.
// Delete when the bounded collector contract is retired or subsumed.
#[test]
fn a_store_that_never_settles_exhausts_the_limit() -> Result<(), Box<dyn Error>> {
    let mut source = Scripted {
        coordinates: (1..=4)
            .map(|generation| published(generation, [0; 32]).map(Ok))
            .collect::<Result<_, _>>()?,
        views: [Ok("superseded one"), Ok("superseded two")].into(),
    };
    let limit = ReaderAttemptLimit::new(NonZeroU32::new(2).ok_or("zero")?);
    assert!(matches!(
        collect_retention_view(&mut source, limit),
        Err(RetentionViewError::AttemptsExhausted { attempts: 2 })
    ));
    Ok(())
}

// Size: small. Oracle: an absent catalog cannot produce a reader view.
// Delete when catalog absence is removed from the contract or subsumed.
#[test]
fn an_absent_catalog_refuses_collection() {
    let mut source = Scripted {
        coordinates: [Ok(ABSENT)].into(),
        views: [Ok("unselected payload")].into(),
    };
    assert!(matches!(
        collect_retention_view(&mut source, ReaderAttemptLimit::DEFAULT),
        Err(RetentionViewError::CatalogAbsent)
    ));
}

fn require_io(mut source: Scripted) -> Result<(), Box<dyn Error>> {
    let error = collect_retention_view(&mut source, ReaderAttemptLimit::DEFAULT)
        .err()
        .ok_or("read failure returned a view")?;
    let RetentionViewError::Io { source } = error else {
        return Err(format!("source I/O failure was reclassified: {error:?}").into());
    };
    assert_eq!(
        source.raw_os_error(),
        Some(123),
        "collector must preserve the original source I/O error"
    );
    Ok(())
}

// Size: small. Oracle: initial-coordinate I/O failure is returned intact.
// Delete when the source port is removed or stronger propagation coverage subsumes it.
#[test]
fn initial_coordinate_failure_preserves_its_io_cause() -> Result<(), Box<dyn Error>> {
    require_io(Scripted {
        coordinates: [Err(io::Error::from_raw_os_error(123))].into(),
        views: [Ok("unselected payload")].into(),
    })
}

// Size: small. Oracle: load I/O failure is returned intact.
// Delete when the source port is removed or stronger propagation coverage subsumes it.
#[test]
fn view_load_failure_preserves_its_io_cause() -> Result<(), Box<dyn Error>> {
    require_io(Scripted {
        coordinates: [Ok(published(1, [0; 32])?)].into(),
        views: [Err(io::Error::from_raw_os_error(123))].into(),
    })
}

// Size: small. Oracle: final-coordinate I/O failure invalidates the loaded view.
// Delete when the source port is removed or stronger propagation coverage subsumes it.
#[test]
fn final_coordinate_failure_preserves_its_io_cause() -> Result<(), Box<dyn Error>> {
    require_io(Scripted {
        coordinates: [
            Ok(published(1, [0; 32])?),
            Err(io::Error::from_raw_os_error(123)),
        ]
        .into(),
        views: [Ok("unverified payload")].into(),
    })
}
