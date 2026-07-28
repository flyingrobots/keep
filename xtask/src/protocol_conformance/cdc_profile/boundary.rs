//! This module owns CDC boundary-table admission and per-case verification.

mod named_laws;
mod schedule;

use std::collections::{BTreeMap, BTreeSet};

use crate::protocol_conformance::ConformanceError;
use crate::protocol_conformance::canonical::{case_name, decimal};
use crate::protocol_conformance::cdc_profile::scalar_fastcdc::{
    StreamingChunker, reference_boundaries,
};
use crate::protocol_conformance::cdc_profile::source::Sources;
use crate::protocol_conformance::cdc_profile::{GearTable, MAXIMUM, MINIMUM};
use crate::protocol_conformance::corpus::{Corpus, TablePolicy};

const BOUNDARY_COLUMNS: [&str; 3] = ["case", "chunk_count", "boundaries"];
const BOUNDARY_POLICY: TablePolicy =
    TablePolicy::new("keep.cdc-boundaries/v1", &BOUNDARY_COLUMNS, 1_048_576, 256);
const MAX_BOUNDARIES: usize = 2_097_152;

pub(super) struct Boundaries {
    values: BTreeMap<String, Vec<usize>>,
}

#[derive(Clone, Copy)]
struct StreamCase<'a> {
    name: &'a str,
    source: &'a [u8],
    expected: &'a [usize],
    gear: &'a GearTable,
}

impl Boundaries {
    pub(super) fn get(&self, name: &str) -> Result<&[usize], ConformanceError> {
        self.values.get(name).map(Vec::as_slice).ok_or_else(|| {
            ConformanceError::violation(format!("CDC boundary case is absent: {name}"))
        })
    }
}

pub(super) fn read(corpus: &Corpus, sources: &Sources) -> Result<Boundaries, ConformanceError> {
    let source_lengths = sources
        .iter()
        .map(|(name, source)| (name.to_owned(), source.len()))
        .collect();
    read_values(corpus, &source_lengths)
}

fn read_values(
    corpus: &Corpus,
    source_lengths: &BTreeMap<String, usize>,
) -> Result<Boundaries, ConformanceError> {
    let mut values = BTreeMap::new();
    for row in corpus.rows("boundaries.tsv", BOUNDARY_POLICY)? {
        let name = case_name(row.field("case")?, "boundaries.tsv")?;
        if values.contains_key(name) {
            return Err(ConformanceError::violation(format!(
                "boundaries.tsv: duplicate case {name:?}"
            )));
        }
        if !source_lengths.contains_key(name) {
            return Err(ConformanceError::violation(format!(
                "boundaries.tsv: case is outside the required exact source set: {name:?}"
            )));
        }
        let source_length = source_lengths.get(name).copied().ok_or_else(|| {
            ConformanceError::violation(format!("boundary source length is absent: {name}"))
        })?;
        let count = decimal(
            row.field("chunk_count")?,
            &format!("{name} chunk count"),
            MAX_BOUNDARIES,
        )?;
        let ends = parse_ends(row.field("boundaries")?, name, source_length)?;
        if ends.len() != count {
            return Err(ConformanceError::violation(format!(
                "{name}: declared {count} chunks, recorded {}",
                ends.len()
            )));
        }
        values.insert(name.to_owned(), ends);
    }
    if values.keys().ne(source_lengths.keys()) {
        return Err(ConformanceError::violation(
            "boundary cases differ from the required exact source set",
        ));
    }
    Ok(Boundaries { values })
}

fn parse_ends(
    value: &str,
    name: &str,
    source_length: usize,
) -> Result<Vec<usize>, ConformanceError> {
    if value == "-" {
        return Ok(Vec::new());
    }
    value
        .split(',')
        .map(|boundary| decimal(boundary, &format!("{name} boundary"), source_length))
        .collect()
}

pub(super) fn validate(
    sources: &Sources,
    boundaries: &Boundaries,
    gear: &GearTable,
) -> Result<(), ConformanceError> {
    for (name, source) in sources.iter() {
        validate_case(name, source, boundaries.get(name)?, gear)?;
    }
    named_laws::validate(sources, boundaries, gear)
}

fn validate_case(
    name: &str,
    source: &[u8],
    expected: &[usize],
    gear: &GearTable,
) -> Result<(), ConformanceError> {
    if expected != reference_boundaries(source, gear)? {
        return Err(ConformanceError::violation(format!(
            "{name}: expected boundaries differ from scalar Gear64/FastCDC"
        )));
    }
    validate_partition(name, source, expected)?;
    validate_streaming(name, source, expected, gear)
}

fn validate_partition(
    name: &str,
    source: &[u8],
    expected: &[usize],
) -> Result<(), ConformanceError> {
    let mut previous = 0_usize;
    let mut reconstructed = Vec::with_capacity(source.len());
    for (index, boundary) in expected.iter().copied().enumerate() {
        if boundary <= previous || boundary > source.len() {
            return Err(ConformanceError::violation(format!(
                "{name}: boundaries are not strictly increasing and bounded"
            )));
        }
        let size = boundary
            .checked_sub(previous)
            .ok_or_else(|| ConformanceError::violation(format!("{name}: chunk size underflow")))?;
        let has_following = index
            .checked_add(1)
            .is_some_and(|next| next < expected.len());
        if size > MAXIMUM || (has_following && size < MINIMUM) {
            return Err(ConformanceError::violation(format!(
                "{name}: chunk size {size} violates profile bounds"
            )));
        }
        reconstructed.extend_from_slice(source.get(previous..boundary).ok_or_else(|| {
            ConformanceError::violation(format!("{name}: chunk range is outside source"))
        })?);
        previous = boundary;
    }
    let coverage_moved = (!source.is_empty() && (expected.is_empty() || previous != source.len()))
        || (source.is_empty() && !expected.is_empty());
    if coverage_moved || reconstructed != source {
        return Err(ConformanceError::violation(format!(
            "{name}: boundaries do not reconstruct the source exactly"
        )));
    }
    Ok(())
}

fn validate_streaming(
    name: &str,
    source: &[u8],
    expected: &[usize],
    gear: &GearTable,
) -> Result<(), ConformanceError> {
    let case = StreamCase {
        name,
        source,
        expected,
        gear,
    };
    run_whole(case, 0)?;
    run_sizes(case, &[4_093], 1)?;
    run_sizes(case, &[1, 7, 257, 4_093, 65_521], 2)?;
    run_adjacent(case, 3)?;
    if source.len() <= MINIMUM + 1 {
        run_sizes(case, &[1], 4)?;
    }
    if name == "probe-byte-carry" {
        run_sizes(case, &[1], 5)?;
    }
    Ok(())
}

fn run_whole(case: StreamCase<'_>, schedule: usize) -> Result<(), ConformanceError> {
    let mut chunker = StreamingChunker::new(case.gear);
    chunker.feed(case.source)?;
    admit_stream(case, schedule, &mut chunker)
}

fn run_sizes(
    case: StreamCase<'_>,
    sizes: &[usize],
    schedule: usize,
) -> Result<(), ConformanceError> {
    let mut chunker = StreamingChunker::new(case.gear);
    self::schedule::feed_sizes(&mut chunker, case.source, sizes)?;
    admit_stream(case, schedule, &mut chunker)
}

fn run_adjacent(case: StreamCase<'_>, schedule: usize) -> Result<(), ConformanceError> {
    let mut points = BTreeSet::from([0, case.source.len()]);
    for boundary in case.expected {
        points.insert(boundary.saturating_sub(1));
        points.insert(*boundary);
        points.insert(boundary.saturating_add(1).min(case.source.len()));
    }
    let mut chunker = StreamingChunker::new(case.gear);
    let points = points.into_iter().collect::<Vec<_>>();
    for pair in points.windows(2) {
        let [left, right] = pair else {
            return Err(ConformanceError::violation(
                "boundary-adjacent pair has the wrong width",
            ));
        };
        if left < right {
            chunker.feed(case.source.get(*left..*right).ok_or_else(|| {
                ConformanceError::violation("boundary-adjacent range is outside its source")
            })?)?;
        }
    }
    admit_stream(case, schedule, &mut chunker)
}

fn admit_stream(
    case: StreamCase<'_>,
    schedule: usize,
    chunker: &mut StreamingChunker<'_>,
) -> Result<(), ConformanceError> {
    chunker.finish()?;
    if chunker.boundaries()? != case.expected || chunker.reconstruct() != case.source {
        return Err(ConformanceError::violation(format!(
            "{}: partition schedule {schedule} moved boundaries",
            case.name
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
