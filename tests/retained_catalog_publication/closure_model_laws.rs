//! Exhaustive comparison of the gate with a boring retained-closure model.

use std::error::Error;

use keep::{
    AdmittedRetentionRoot, RetainedCatalogPublicationError, RetentionClosureVerificationError,
    RetentionManifestEntry, SegmentRecordIdentity,
};

use crate::catalog_fixture::{Candidate, dropped_identity, with_candidate};
use crate::model_storage::{EvidenceRead, ModelStorage};
use crate::publish;
use crate::retention_fixture::{RetentionRecords, RootShape};

const RETAIN: RootShape = RootShape::Retaining;
const RELEASE: RootShape = RootShape::Released;

/// Typed result of one publication call.
#[derive(Debug, Eq, PartialEq)]
enum Outcome {
    /// Published, preserving this many manifest roots, or no retention head.
    Published(Option<u32>),
    /// Refused at this manifest entry for this missing closure member.
    Refused(RetentionManifestEntry, SegmentRecordIdentity),
}

/// Outcome, catalog stage count, and every evidence read of one call.
#[derive(Debug, Eq, PartialEq)]
struct Observation {
    outcome: Outcome,
    stages: usize,
    roots_read: Vec<RetentionManifestEntry>,
}

fn configurations() -> Result<Vec<RetentionRecords>, Box<dyn Error>> {
    Ok(vec![
        RetentionRecords::unretained(),
        RetentionRecords::golden()?,
        RetentionRecords::built(&[(b"a", RELEASE)])?,
        RetentionRecords::built(&[(b"a", RETAIN), (b"b", RELEASE)])?,
        RetentionRecords::built(&[(b"a", RETAIN), (b"b", RETAIN)])?,
        RetentionRecords::built(&[(b"a", RELEASE), (b"b", RELEASE), (b"c", RETAIN)])?,
    ])
}

#[test]
fn gate_agrees_with_the_model_for_every_candidate_and_root_set() -> Result<(), Box<dyn Error>> {
    let mut cases = 0_u32;
    for records in configurations()? {
        for candidate in Candidate::ALL {
            let first = run(&records, candidate)?;
            let second = run(&records, candidate)?;
            assert_eq!(first, second, "verification must be deterministic");
            assert_eq!(first, model(&records, candidate)?, "{candidate:?}");
            cases = cases.checked_add(1).ok_or("case count overflowed")?;
        }
    }
    assert_eq!(cases, 24);
    Ok(())
}

fn run(records: &RetentionRecords, candidate: Candidate) -> Result<Observation, Box<dyn Error>> {
    let mut storage = ModelStorage::new(records.clone());
    let result = with_candidate(candidate, |call| publish(&mut storage, &call))?;
    let outcome = match result {
        Ok(receipt) => Outcome::Published(
            receipt
                .preserved_manifest()
                .map(keep::PreservedRetentionManifest::root_count),
        ),
        Err(error) => refused(&error)?,
    };
    let roots_read = storage
        .reads()
        .iter()
        .filter_map(|(read, _)| match read {
            EvidenceRead::Root(entry) => Some(*entry),
            EvidenceRead::Head | EvidenceRead::Manifest => None,
        })
        .collect();
    Ok(Observation {
        outcome,
        stages: storage.stages().len(),
        roots_read,
    })
}

fn refused(error: &RetainedCatalogPublicationError) -> Result<Outcome, Box<dyn Error>> {
    let RetainedCatalogPublicationError::Closure { entry, source } = error else {
        return Err(format!("unexpected refusal: {error:?}").into());
    };
    let RetentionClosureVerificationError::MissingMember { identity } = source.as_ref() else {
        return Err(format!("unexpected closure refusal: {source:?}").into());
    };
    Ok(Outcome::Refused(*entry, *identity))
}

/// The model: the first retaining root in manifest order refuses any candidate
/// that does not keep the one-zero blob; otherwise every root is verified.
fn model(records: &RetentionRecords, candidate: Candidate) -> Result<Observation, Box<dyn Error>> {
    let entries = match records.head {
        Some(_) => records.entries()?,
        None => Vec::new(),
    };
    let refusal =
        first_retaining(records, &entries)?.filter(|_| candidate != Candidate::Preserving);
    let Some(position) = refusal else {
        let count = records
            .head
            .as_ref()
            .map(|_| u32::try_from(entries.len()))
            .transpose()?;
        return Ok(Observation {
            outcome: Outcome::Published(count),
            stages: crate::catalog_only_phases().len(),
            roots_read: entries,
        });
    };
    let entry = *entries
        .get(position)
        .ok_or("model position is out of range")?;
    let read = position.checked_add(1).ok_or("model position overflowed")?;
    Ok(Observation {
        outcome: Outcome::Refused(entry, dropped_identity(candidate)?),
        stages: 0,
        roots_read: entries.into_iter().take(read).collect(),
    })
}

fn first_retaining(
    records: &RetentionRecords,
    entries: &[RetentionManifestEntry],
) -> Result<Option<usize>, Box<dyn Error>> {
    for (position, entry) in entries.iter().enumerate() {
        let (_, bytes) = records
            .roots
            .iter()
            .find(|(digest, _)| *digest == entry.root_digest())
            .ok_or("records omit a manifest root")?;
        if AdmittedRetentionRoot::decode(bytes)?.root().anchor_count() > 0 {
            return Ok(Some(position));
        }
    }
    Ok(None)
}
