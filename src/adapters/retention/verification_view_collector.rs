//! This module owns bounded conflicting observations for verification collection.
#![expect(
    clippy::result_large_err,
    reason = "preserve bounded semantic refusal coordinates"
)]

use super::{
    ReaderAttemptLimit, RetentionViewCoordinates, RetentionViewError, RetentionViewSource,
    collect_retention_view,
};
use crate::{VerificationError, VerificationRefusal, VerificationSource, VerificationSubject};
use std::io;

/// Collects a consistent view or retains the last conflicting observation pair.
///
/// Uses the same before/load/after algorithm and attempt bound as
/// `collect_retention_view`. Tracking uses constant memory; only an error
/// allocates its boxed cause and, for ambiguity, one pair of coordinates.
/// It performs only the supplied source's observations and never repairs or
/// writes. A conflict can be ordinary concurrent publication, not corruption.
///
/// # Errors
///
/// Returns missing catalog evidence, the exact last pair when no attempt
/// agrees, or an operational observation error with its original typed cause.
/// An error returns no partially collected view.
pub fn collect_verification_view<S: RetentionViewSource>(
    source: &mut S,
    limit: ReaderAttemptLimit,
) -> Result<S::View, VerificationError> {
    let mut observed = ObservedSource {
        source,
        previous: None,
        last: None,
    };
    collect_retention_view(&mut observed, limit)
        .map_err(|source| classify(source, observed.previous, observed.last))
}

struct ObservedSource<'a, S> {
    source: &'a mut S,
    previous: Option<RetentionViewCoordinates>,
    last: Option<RetentionViewCoordinates>,
}

impl<S: RetentionViewSource> RetentionViewSource for ObservedSource<'_, S> {
    type View = S::View;

    fn coordinates(&mut self) -> io::Result<RetentionViewCoordinates> {
        let coordinates = self.source.coordinates()?;
        self.previous = self.last.replace(coordinates);
        Ok(coordinates)
    }

    fn load(&mut self) -> io::Result<Self::View> {
        self.source.load()
    }
}

fn classify(
    source: RetentionViewError,
    before: Option<RetentionViewCoordinates>,
    after: Option<RetentionViewCoordinates>,
) -> VerificationError {
    let observed_refusal = match &source {
        RetentionViewError::Io { source } => super::verification_observation_error::refusal(source),
        _ => None,
    };
    if let Some(refusal) = observed_refusal {
        return VerificationError::Refused {
            refusal,
            source: Some(Box::new(VerificationSource::View(source))),
        };
    }
    let refusal = match (&source, before, after) {
        (RetentionViewError::CatalogAbsent, _, _) => VerificationRefusal::Missing {
            subject: VerificationSubject::PublishedCatalog,
        },
        (RetentionViewError::AttemptsExhausted { .. }, Some(before), Some(after)) => {
            VerificationRefusal::Ambiguous {
                candidates: Box::new([before, after]),
            }
        }
        _ => {
            return VerificationError::Operational {
                source: Box::new(VerificationSource::View(source)),
            };
        }
    };
    VerificationError::Refused {
        refusal,
        source: Some(Box::new(VerificationSource::View(source))),
    }
}
