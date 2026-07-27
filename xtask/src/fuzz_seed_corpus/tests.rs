//! This module owns deterministic Golden File Worldline seed evidence.

use std::path::Path;

use super::{FuzzSeedError, Seed, filesystem::RepositoryFiles, golden_protocol_seeds, prepare};

const TABLES: [&str; 5] = [
    "identities.tsv",
    "invalid-text.tsv",
    "mutations.tsv",
    "steps.tsv",
    "capabilities.tsv",
];

#[test]
fn golden_protocol_seeds_reach_every_production_parser() -> Result<(), FuzzSeedError> {
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or(FuzzSeedError::RepositoryRoot)?;
    let seeds = golden_protocol_seeds(repository_root)?;
    let selectors = seeds
        .iter()
        .filter_map(|seed| seed.content.first().copied())
        .collect::<Vec<_>>();
    assert_eq!(selectors, (0_u8..=8).collect::<Vec<_>>());
    assert!(seeds.iter().all(|seed| seed.target == "golden_protocol"));
    Ok(())
}

#[test]
fn fuzz_workflows_delegate_seed_preparation_to_the_rust_task() -> Result<(), FuzzSeedError> {
    use std::fs;

    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or(FuzzSeedError::RepositoryRoot)?;
    for relative in [
        ".github/workflows/ci.yml",
        ".github/workflows/fuzz-scheduled.yml",
    ] {
        let path = repository_root.join(relative);
        let workflow = fs::read_to_string(&path)
            .map_err(|source| FuzzSeedError::io("read fuzz workflow", &path, source))?;
        assert_eq!(
            workflow
                .matches("run: cargo xtask prepare-fuzz-corpus")
                .count(),
            1
        );
        assert!(!workflow.contains("python3 fuzz/prepare_corpus.py"));
    }
    Ok(())
}

#[test]
fn fuzz_seed_diagnostics_escape_terminal_controls() {
    let diagnostic =
        FuzzSeedError::violation("first\nError: forged\rrewrite\u{1b}[31m").to_string();
    assert_eq!(
        diagnostic,
        "fuzz seed preparation failed: first\\nError: forged\\rrewrite\\u{1b}[31m"
    );
    assert_eq!(diagnostic.lines().count(), 1);
}

#[test]
fn seed_preparation_materializes_the_complete_deterministic_set() -> Result<(), FuzzSeedError> {
    use std::env;
    use std::fs;
    use std::process;

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or(FuzzSeedError::RepositoryRoot)?;
    let root = env::temp_dir().join(format!("keep-fuzz-seeds-{}", process::id()));
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

    prepare(&root)?;
    let corpus = root.join("fuzz/corpus");
    let first = seed_contents(&corpus)?;
    assert_eq!(first.len(), 22);
    assert_eq!(
        first
            .keys()
            .filter(|name| name.starts_with("golden_protocol/"))
            .count(),
        9
    );
    prepare(&root)?;
    assert_eq!(seed_contents(&corpus)?, first);

    fs::remove_dir_all(&root)
        .map_err(|source| FuzzSeedError::io("remove test seed root", &root, source))?;
    Ok(())
}

#[test]
fn seed_publication_does_not_mutate_a_hard_link_target() -> Result<(), FuzzSeedError> {
    use std::env;
    use std::fs;
    use std::process;

    let root = env::temp_dir().join(format!("keep-fuzz-seed-hard-link-{}", process::id()));
    let target = root.join("fuzz/corpus/blob_hasher");
    fs::create_dir_all(&target)
        .map_err(|source| FuzzSeedError::io("create test corpus", &target, source))?;
    let protected = root.join("protected");
    fs::write(&protected, b"authoritative")
        .map_err(|source| FuzzSeedError::io("write protected test file", &protected, source))?;
    let destination = target.join("empty");
    fs::hard_link(&protected, &destination)
        .map_err(|source| FuzzSeedError::io("link test seed destination", &destination, source))?;

    RepositoryFiles::open(&root)?.write_seeds(&[Seed {
        target: "blob_hasher",
        name: "empty",
        content: b"derived".to_vec(),
    }])?;
    let protected_content = fs::read(&protected)
        .map_err(|source| FuzzSeedError::io("read protected test file", &protected, source))?;
    let seed_content = fs::read(&destination)
        .map_err(|source| FuzzSeedError::io("read test seed", &destination, source))?;
    fs::remove_dir_all(&root)
        .map_err(|source| FuzzSeedError::io("remove hard-link test root", &root, source))?;

    assert_eq!(protected_content, b"authoritative");
    assert_eq!(seed_content, b"derived");
    Ok(())
}

fn seed_contents(
    root: &Path,
) -> Result<std::collections::BTreeMap<String, Vec<u8>>, FuzzSeedError> {
    let mut contents = std::collections::BTreeMap::new();
    collect_seed_contents(root, root, &mut contents)?;
    Ok(contents)
}

fn collect_seed_contents(
    root: &Path,
    directory: &Path,
    contents: &mut std::collections::BTreeMap<String, Vec<u8>>,
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
