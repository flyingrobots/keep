//! Deterministic verification collection laws at the observation port.

mod support;
use keep::{
    CatalogGeneration, ChecksummedPublicationHead, ChecksummedRetentionHead, ReaderAttemptLimit,
    RetentionViewCoordinates, RetentionViewError, RetentionViewSource, VerificationError,
    VerificationRefusal, VerificationSource, VerificationSubject, collect_verification_view,
};
use std::collections::VecDeque;
use std::error::Error;
use std::io;

// Size: small. Oracle: a reportable view must belong to agreeing observations.
// This is port-level deterministic scheduling, not filesystem interleaving proof.
// Delete when a stronger generated collection model subsumes these schedules.
#[test]
fn changing_catalogs_discard_the_superseded_view() -> Result<(), Box<dyn Error>> {
    let first = coordinates(1)?;
    let second = coordinates(2)?;
    let mut source = Views {
        coordinates: [Ok(first), Ok(second), Ok(second), Ok(second)].into(),
        values: ["superseded", "stable"].into(),
    };
    assert_eq!(
        collect_verification_view(&mut source, ReaderAttemptLimit::DEFAULT)?,
        "stable",
        "only the agreeing observation may supply the returned view"
    );
    Ok(())
}

// Size: small. Oracle: exhausted collection retains the actual last pair, not
// the first pair, an invented candidate, a partial view or a corruption claim.
// Delete when a stronger bounded history model includes this exact schedule.
#[test]
fn exhausted_collection_reports_exact_conflicting_candidates() -> Result<(), Box<dyn Error>> {
    let [first, second, third, fourth] = [
        coordinates(1)?,
        coordinates(2)?,
        coordinates(3)?,
        coordinates(4)?,
    ];
    let mut source = Views {
        coordinates: [Ok(first), Ok(second), Ok(third), Ok(fourth)].into(),
        values: ["first rejected view", "second rejected view"].into(),
    };
    let error = collect_verification_view(
        &mut source,
        ReaderAttemptLimit::new(std::num::NonZeroU32::new(2).ok_or("zero attempts")?),
    )
    .err()
    .ok_or("moving view certified")?;
    let VerificationError::Refused {
        refusal: VerificationRefusal::Ambiguous { candidates },
        source: Some(source),
    } = error
    else {
        return Err("conflicting observations lost ambiguity evidence".into());
    };
    assert_eq!(
        *candidates,
        [third, fourth],
        "last actual observations are the bounded conflict witness"
    );
    assert!(
        matches!(
            *source,
            VerificationSource::View(RetentionViewError::AttemptsExhausted { attempts: 2 })
        ),
        "original collection limit must survive"
    );
    Ok(())
}

// Size: small. Oracle: retention publication is part of the complete view.
// Delete when a generated collection law subsumes head appearance/disappearance.
#[test]
fn retention_head_changes_are_not_hidden_by_a_stable_catalog() -> Result<(), Box<dyn Error>> {
    let before = coordinates(1)?;
    let bytes = support::decode_hex(
        include_str!("../conformance/segment-store/v2/one-root-head.hex").trim_end(),
    )?;
    let after = RetentionViewCoordinates {
        retention: Some(*ChecksummedRetentionHead::decode(&bytes)?.head()),
        ..before
    };
    let mut source = Views {
        coordinates: [Ok(before), Ok(after)].into(),
        values: ["unbound"].into(),
    };
    let error = collect_verification_view(
        &mut source,
        ReaderAttemptLimit::new(std::num::NonZeroU32::MIN),
    )
    .err()
    .ok_or("retention change hidden")?;
    assert!(
        matches!(error, VerificationError::Refused { refusal: VerificationRefusal::Ambiguous { ref candidates }, .. } if **candidates == [before, after]),
        "exact full-head disagreement required: {error:?}"
    );
    Ok(())
}

// Size: small. Oracle: failed observation does not prove absence or corruption.
// Delete when a broader operational-source conformance law subsumes it.
#[test]
fn an_observation_failure_preserves_its_operational_cause() -> Result<(), Box<dyn Error>> {
    let mut source = Views {
        coordinates: [Err(io::Error::from(io::ErrorKind::PermissionDenied))].into(),
        values: [].into(),
    };
    let error = collect_verification_view(&mut source, ReaderAttemptLimit::DEFAULT)
        .err()
        .ok_or("unreadable view certified")?;
    assert!(
        matches!(error, VerificationError::Operational { source } if matches!(source.as_ref(), VerificationSource::View(RetentionViewError::Io { source }) if source.kind() == io::ErrorKind::PermissionDenied)),
        "permission failure must remain operational"
    );
    Ok(())
}

// Size: small. Oracle: an observed absent publication head is exact absence.
// Delete when publication permits a headless catalog as a complete view.
#[test]
fn no_published_catalog_is_missing_evidence() -> Result<(), Box<dyn Error>> {
    let mut source = Views {
        coordinates: [Ok(RetentionViewCoordinates {
            catalog: None,
            retention: None,
        })]
        .into(),
        values: [].into(),
    };
    let error = collect_verification_view(&mut source, ReaderAttemptLimit::DEFAULT)
        .err()
        .ok_or("headless view certified")?;
    assert!(
        matches!(error, VerificationError::Refused { refusal: VerificationRefusal::Missing { subject: VerificationSubject::PublishedCatalog }, source: Some(source) } if matches!(*source, VerificationSource::View(RetentionViewError::CatalogAbsent))),
        "missing head must retain its exact cause"
    );
    Ok(())
}

fn coordinates(generation: u64) -> Result<RetentionViewCoordinates, Box<dyn Error>> {
    let bytes = support::decode_hex(
        include_str!("../conformance/segment-store/v1/one-zero-head.hex").trim_end(),
    )?;
    let head = ChecksummedPublicationHead::decode(&bytes)?;
    Ok(RetentionViewCoordinates {
        catalog: Some((
            CatalogGeneration::new(generation)?,
            head.catalog_length(),
            head.catalog_digest(),
        )),
        retention: None,
    })
}

struct Views {
    coordinates: VecDeque<io::Result<RetentionViewCoordinates>>,
    values: VecDeque<&'static str>,
}
impl RetentionViewSource for Views {
    type View = &'static str;
    fn coordinates(&mut self) -> io::Result<RetentionViewCoordinates> {
        self.coordinates
            .pop_front()
            .ok_or_else(|| io::Error::other("observation schedule exhausted"))?
    }
    fn load(&mut self) -> io::Result<Self::View> {
        self.values
            .pop_front()
            .ok_or_else(|| io::Error::other("view schedule exhausted"))
    }
}

// Size: small. Oracle: the decoder's precise version contradiction is corruption,
// and remains reachable as the original typed cause after view collection.
// Delete when a stronger public ingress matrix covers this observation path.
#[test]
fn malformed_publication_observations_preserve_the_decoder_contradiction()
-> Result<(), Box<dyn Error>> {
    let cause = keep::PublicationHeadDecodeError::UnsupportedVersion {
        expected: 1,
        observed: 2,
    };
    let mut source = Views {
        coordinates: [Err(io::Error::new(io::ErrorKind::InvalidData, cause))].into(),
        values: [].into(),
    };
    let error = collect_verification_view(&mut source, ReaderAttemptLimit::DEFAULT)
        .err()
        .ok_or("malformed publication certified")?;
    let VerificationError::Refused {
        refusal:
            VerificationRefusal::Corrupt {
                subject: VerificationSubject::PublishedCatalog,
                ..
            },
        source: Some(source),
    } = error
    else {
        return Err("known contradiction lost its corruption classification".into());
    };
    let VerificationSource::View(RetentionViewError::Io { source }) = *source else {
        return Err("original observation boundary lost".into());
    };
    assert!(
        matches!(
            source
                .get_ref()
                .and_then(|source| source.downcast_ref::<keep::PublicationHeadDecodeError>()),
            Some(keep::PublicationHeadDecodeError::UnsupportedVersion {
                expected: 1,
                observed: 2
            })
        ),
        "exact version contradiction must survive"
    );
    Ok(())
}
