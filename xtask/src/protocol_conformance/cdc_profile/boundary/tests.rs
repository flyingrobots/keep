//! This module owns CDC boundary admission-order and arithmetic evidence.

use std::collections::BTreeMap;
use std::fs;

use super::schedule::{feed_sizes_with, partition_end};
use super::{ConformanceError, Corpus, StreamingChunker, read_values};
use crate::test_directory::TestDirectory;

#[test]
fn partition_offset_overflow_is_refused() {
    assert!(matches!(
        partition_end(
            usize::MAX,
            1,
            usize::MAX,
            "partition schedule",
        ),
        Err(ConformanceError::Violation(ref message))
            if message == "partition schedule offset overflow"
    ));
}

#[test]
fn schedule_observer_precedes_every_partition() -> Result<(), ConformanceError> {
    let gear = [0_u64; 256];
    let mut chunker = StreamingChunker::new(&gear);
    let mut observations = 0_usize;
    feed_sizes_with(&mut chunker, b"abc", &[1, 2], "test schedule", |_| {
        observations = observations
            .checked_add(1)
            .ok_or_else(|| ConformanceError::violation("observer count overflow"))?;
        Ok(())
    })?;
    assert_eq!(observations, 2);
    Ok(())
}

#[test]
fn boundary_case_set_precedes_case_semantics() -> Result<(), ConformanceError> {
    let directory = TestDirectory::create("cdc-boundary-case-set").map_err(|source| {
        ConformanceError::io("create boundary test corpus", "temporary", source)
    })?;
    let root = directory.path().to_owned();
    let path = root.join("boundaries.tsv");
    fs::write(
        &path,
        "keep.cdc-boundaries/v1\n\
         case\tchunk_count\tboundaries\n\
         rogue\tmalformed\tmalformed\n",
    )
    .map_err(|source| ConformanceError::io("write boundary test corpus", &path, source))?;
    let source_lengths = BTreeMap::from([(String::from("known"), 1)]);

    let result = read_values(&Corpus::open(root.clone())?, &source_lengths);
    assert!(matches!(
        result,
        Err(ConformanceError::Violation(ref message))
            if message
                == "boundaries.tsv: case is outside the required exact source set: \"rogue\""
    ));
    directory
        .close()
        .map_err(|source| ConformanceError::io("remove boundary test corpus", &root, source))?;
    Ok(())
}

#[test]
fn duplicate_boundary_case_precedes_later_case_semantics() -> Result<(), ConformanceError> {
    let directory = TestDirectory::create("cdc-boundary-duplicate").map_err(|source| {
        ConformanceError::io("create boundary test corpus", "temporary", source)
    })?;
    let root = directory.path().to_owned();
    let path = root.join("boundaries.tsv");
    fs::write(
        &path,
        "keep.cdc-boundaries/v1\n\
         case\tchunk_count\tboundaries\n\
         known\t1\t1\n\
         known\tmalformed\tmalformed\n",
    )
    .map_err(|source| ConformanceError::io("write boundary test corpus", &path, source))?;
    let source_lengths = BTreeMap::from([(String::from("known"), 1)]);

    let result = read_values(&Corpus::open(root.clone())?, &source_lengths);
    assert!(matches!(
        result,
        Err(ConformanceError::Violation(ref message))
            if message == "boundaries.tsv: duplicate case \"known\""
    ));
    directory
        .close()
        .map_err(|source| ConformanceError::io("remove boundary test corpus", &root, source))?;
    Ok(())
}
