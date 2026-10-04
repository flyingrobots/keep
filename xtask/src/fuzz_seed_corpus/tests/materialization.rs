//! Deterministic recovery-counterexample materialization evidence.

use std::collections::BTreeMap;
use std::path::Path;

use super::super::{
    FuzzSeedError, catalog_seeds, gc_seeds, layout_seeds, migration_seeds, prepare,
    retention_seeds, segment_seeds, verification_seeds,
};
use crate::test_directory::TestDirectory;

const TABLES: [&str; 5] = [
    "identities.tsv",
    "invalid-text.tsv",
    "mutations.tsv",
    "steps.tsv",
    "capabilities.tsv",
];

// Size: medium. Oracle: emitted version-2 partial seal triggers runtime refusal.
// Delete if stronger emitted-artifact replay subsumes this stable counterexample.
#[test]
fn seed_preparation_preserves_a_replayable_recovery_counterexample()
-> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or(FuzzSeedError::RepositoryRoot)?;
    let directory = TestDirectory::create("fuzz-seeds")?;
    let root = directory.path();
    let conformance = root.join("conformance/golden-file-worldline/v1");
    fs::create_dir_all(&conformance).map_err(|source| {
        FuzzSeedError::io("create test conformance root", &conformance, source)
    })?;
    for table in TABLES {
        let source_path = source_root
            .join("conformance/golden-file-worldline/v1")
            .join(table);
        let destination = conformance.join(table);
        fs::copy(&source_path, &destination)
            .map_err(|source| FuzzSeedError::io("copy test table", &destination, source))?;
    }
    copy_layout_fixtures(source_root, root)?;
    copy_segment_fixtures(source_root, root)?;
    copy_catalog_fixtures(source_root, root)?;
    copy_version_two_fixtures(source_root, root)?;
    copy_verification_fixtures(source_root, root)?;

    prepare(root)?;
    let corpus = root.join("fuzz/corpus");
    let first = seed_contents(&corpus)?;
    verify_recovery_counterexample(&first)?;
    prepare(root)?;
    assert_eq!(seed_contents(&corpus)?, first);

    directory.close()?;
    Ok(())
}

fn copy_layout_fixtures(source_root: &Path, root: &Path) -> Result<(), FuzzSeedError> {
    use std::fs;

    let layout_directory = root.join("conformance/layout/v1");
    fs::create_dir_all(&layout_directory).map_err(|source| {
        FuzzSeedError::io(
            "create test layout conformance root",
            &layout_directory,
            source,
        )
    })?;
    for fixture in layout_seeds::FIXTURES {
        let source_path = source_root.join("conformance/layout/v1").join(fixture);
        let destination = layout_directory.join(fixture);
        fs::copy(&source_path, &destination)
            .map_err(|source| FuzzSeedError::io("copy test layout", &destination, source))?;
    }
    Ok(())
}

fn copy_segment_fixtures(source_root: &Path, root: &Path) -> Result<(), FuzzSeedError> {
    use std::fs;

    let segment_directory = root.join("conformance/segment-store/v1");
    fs::create_dir_all(&segment_directory).map_err(|source| {
        FuzzSeedError::io(
            "create test segment conformance root",
            &segment_directory,
            source,
        )
    })?;
    for fixture in segment_seeds::FIXTURES {
        let source_path = source_root
            .join("conformance/segment-store/v1")
            .join(fixture);
        let destination = segment_directory.join(fixture);
        fs::copy(&source_path, &destination)
            .map_err(|source| FuzzSeedError::io("copy test segment", &destination, source))?;
    }
    Ok(())
}

fn copy_catalog_fixtures(source_root: &Path, root: &Path) -> Result<(), FuzzSeedError> {
    use std::fs;

    let catalog_directory = root.join("conformance/segment-store/v1");
    for (_selector, fixture) in catalog_seeds::FIXTURES {
        let source_path = source_root
            .join("conformance/segment-store/v1")
            .join(fixture);
        let destination = catalog_directory.join(fixture);
        fs::copy(&source_path, &destination)
            .map_err(|source| FuzzSeedError::io("copy test catalog", &destination, source))?;
    }
    Ok(())
}

fn copy_verification_fixtures(source_root: &Path, root: &Path) -> Result<(), FuzzSeedError> {
    use std::fs;

    let directory = root.join("conformance/verification-receipt/v1");
    fs::create_dir_all(&directory).map_err(|source| {
        FuzzSeedError::io("create test verification-receipt root", &directory, source)
    })?;
    for fixture in verification_seeds::FIXTURES {
        let source_path = source_root
            .join("conformance/verification-receipt/v1")
            .join(fixture);
        let destination = directory.join(fixture);
        fs::copy(&source_path, &destination).map_err(|source| {
            FuzzSeedError::io("copy test verification receipt", &destination, source)
        })?;
    }
    Ok(())
}

fn copy_version_two_fixtures(source_root: &Path, root: &Path) -> Result<(), FuzzSeedError> {
    use std::fs;

    let retention_directory = root.join("conformance/segment-store/v2");
    fs::create_dir_all(&retention_directory).map_err(|source| {
        FuzzSeedError::io(
            "create test version-two conformance root",
            &retention_directory,
            source,
        )
    })?;
    let fixtures = retention_seeds::FIXTURES
        .into_iter()
        .chain(migration_seeds::FIXTURES)
        .chain(gc_seeds::FIXTURES);
    for (_selector, fixture) in fixtures {
        let source_path = source_root
            .join("conformance/segment-store/v2")
            .join(fixture);
        let destination = retention_directory.join(fixture);
        fs::copy(&source_path, &destination)
            .map_err(|source| FuzzSeedError::io("copy test version-two", &destination, source))?;
    }
    Ok(())
}

// Tool output is admitted by the real runtime classifier, not a seed-count oracle.
fn verify_recovery_counterexample(
    contents: &BTreeMap<String, Vec<u8>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = contents
        .get("segment_format/recovery-unsupported-partial-seal-version")
        .ok_or("materialized recovery counterexample is missing")?;
    let (&selector, input) = bytes.split_first().ok_or("counterexample is empty")?;
    assert_eq!(selector, 5, "counterexample must reach the recovery parser");
    let error = keep::classify_recovery_segment_stage(input, keep::SegmentReadPolicy::MAXIMUM)
        .err()
        .ok_or("materialized counterexample did not trigger corruption refusal")?;
    let keep::RecoverySegmentStageError::Seal { source } = error else {
        return Err(format!("wrong materialized corruption boundary: {error:?}").into());
    };
    assert_eq!(
        source,
        keep::SegmentSealError::UnsupportedVersion {
            expected: 1,
            observed: 2
        }
    );
    Ok(())
}

fn seed_contents(root: &Path) -> Result<BTreeMap<String, Vec<u8>>, FuzzSeedError> {
    let mut contents = BTreeMap::new();
    collect_seed_contents(root, root, &mut contents)?;
    Ok(contents)
}

fn collect_seed_contents(
    root: &Path,
    directory: &Path,
    contents: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), FuzzSeedError> {
    use std::fs;

    for entry in fs::read_dir(directory)
        .map_err(|source| FuzzSeedError::io("read test seed directory", directory, source))?
    {
        let entry =
            entry.map_err(|source| FuzzSeedError::io("read test seed entry", directory, source))?;
        let path = entry.path();
        if path.is_dir() {
            collect_seed_contents(root, &path, contents)?;
        } else {
            let name = path
                .strip_prefix(root)
                .map_err(|source| {
                    FuzzSeedError::violation(format!("test seed prefix is invalid: {source}"))
                })?
                .display()
                .to_string();
            let content = fs::read(&path)
                .map_err(|source| FuzzSeedError::io("read test seed", &path, source))?;
            contents.insert(name, content);
        }
    }
    Ok(())
}
