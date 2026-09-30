//! Reader view collection laws against a scripted source.

use std::error::Error;
use std::io;
use std::num::NonZeroU32;

use super::{
    ReaderAttemptLimit, RetentionViewCoordinates, RetentionViewError, RetentionViewSource,
    collect_retention_view,
};
use crate::{CatalogDigest, CatalogGeneration};

struct Scripted {
    coordinates: Vec<RetentionViewCoordinates>,
    loads: u32,
}

impl RetentionViewSource for Scripted {
    type View = u32;

    fn coordinates(&mut self) -> io::Result<RetentionViewCoordinates> {
        if self.coordinates.is_empty() {
            return Err(io::Error::other("script exhausted"));
        }
        Ok(self.coordinates.remove(0))
    }

    fn load(&mut self) -> io::Result<u32> {
        self.loads = self.loads.saturating_add(1);
        Ok(self.loads)
    }
}

fn published(generation: u64) -> Result<RetentionViewCoordinates, Box<dyn Error>> {
    Ok(RetentionViewCoordinates {
        catalog: Some((
            CatalogGeneration::new(generation)?,
            CatalogDigest::from_validated([0; 32]),
        )),
        retention: None,
    })
}

const ABSENT: RetentionViewCoordinates = RetentionViewCoordinates {
    catalog: None,
    retention: None,
};

#[test]
fn a_stable_store_is_collected_on_the_first_attempt() -> Result<(), Box<dyn Error>> {
    let mut source = Scripted {
        coordinates: vec![published(1)?, published(1)?],
        loads: 0,
    };
    let view = collect_retention_view(&mut source, ReaderAttemptLimit::DEFAULT)?;
    assert_eq!(view, 1);
    Ok(())
}

#[test]
fn a_publication_between_the_reads_discards_the_view_and_retries() -> Result<(), Box<dyn Error>> {
    let mut source = Scripted {
        coordinates: vec![published(1)?, published(2)?, published(2)?, published(2)?],
        loads: 0,
    };
    let view = collect_retention_view(&mut source, ReaderAttemptLimit::DEFAULT)?;
    assert_eq!(
        view, 2,
        "the first load was discarded and the second accepted"
    );
    Ok(())
}

#[test]
fn a_store_that_never_settles_exhausts_the_limit() -> Result<(), Box<dyn Error>> {
    let mut source = Scripted {
        coordinates: (1..=8).map(published).collect::<Result<_, _>>()?,
        loads: 0,
    };
    let limit = ReaderAttemptLimit::new(NonZeroU32::new(2).ok_or("zero")?);
    let error = collect_retention_view(&mut source, limit)
        .err()
        .ok_or("a moving store was accepted")?;
    assert!(matches!(
        error,
        RetentionViewError::AttemptsExhausted { attempts: 2 }
    ));
    assert_eq!(source.loads, 2);
    Ok(())
}

#[test]
fn an_absent_catalog_refuses_before_loading() -> Result<(), Box<dyn Error>> {
    let mut source = Scripted {
        coordinates: vec![ABSENT],
        loads: 0,
    };
    let error = collect_retention_view(&mut source, ReaderAttemptLimit::DEFAULT)
        .err()
        .ok_or("an absent catalog was collected")?;
    assert!(matches!(error, RetentionViewError::CatalogAbsent));
    assert_eq!(source.loads, 0);
    Ok(())
}
