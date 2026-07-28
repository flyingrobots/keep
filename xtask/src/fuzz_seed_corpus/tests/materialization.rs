//! Complete deterministic seed materialization evidence.

use std::collections::BTreeMap;
use std::path::Path;

use super::super::{FuzzSeedError, layout_seeds, prepare, segment_seeds};
use crate::test_directory::TestDirectory;

const TABLES: [&str; 5] = [
    "identities.tsv",
    "invalid-text.tsv",
    "mutations.tsv",
    "steps.tsv",
    "capabilities.tsv",
];

#[test]
fn seed_preparation_materializes_the_complete_deterministic_set()
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

    prepare(root)?;
    let corpus = root.join("fuzz/corpus");
    let first = seed_contents(&corpus)?;
    assert_eq!(first.len(), 34);
    assert_eq!(target_seed_count(&first, "golden_protocol/"), 9);
    assert_eq!(target_seed_count(&first, "layout_record/"), 4);
    assert_eq!(target_seed_count(&first, "segment_format/"), 8);
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

fn target_seed_count(contents: &BTreeMap<String, Vec<u8>>, prefix: &str) -> usize {
    contents
        .keys()
        .filter(|name| name.starts_with(prefix))
        .count()
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
