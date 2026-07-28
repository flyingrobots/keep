//! This module owns deterministic Golden File Worldline seed evidence.

use std::path::Path;

use super::{FuzzSeedError, Seed, filesystem::RepositoryFiles, golden_protocol_seeds};
use crate::test_directory::TestDirectory;

mod materialization;

const FILESYSTEM_SOURCE: &str = include_str!("filesystem.rs");
const FUZZ_README: &str = include_str!("../../../fuzz/README.md");

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
fn segment_format_documentation_runs_the_documented_target() {
    assert!(FUZZ_README.contains("fuzz run segment_format"));
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
fn staged_seed_bytes_are_synced_before_publication() {
    let sync = FILESYSTEM_SOURCE.find("file.sync_all()");
    let publish = FILESYSTEM_SOURCE.find(".rename(&temporary, directory, seed.name)");
    assert!(matches!((sync, publish), (Some(sync), Some(publish)) if sync < publish));
    assert!(!FILESYSTEM_SOURCE.contains("file.flush()"));
}

#[test]
fn repository_file_boundary_documents_its_io_contracts() {
    for required in [
        "/// Capability-bound repository access for derived fuzz seed material.",
        "/// Open and retain the repository directory capability.",
        "/// Read one no-follow regular file under an explicit allocation bound.",
        "/// Publish deterministic seeds through synced per-file stages.",
    ] {
        assert!(FILESYSTEM_SOURCE.contains(required));
    }
    assert!(FILESYSTEM_SOURCE.contains("batch-atomic"));
    assert!(FILESYSTEM_SOURCE.contains("The containing directory is not synced"));
}

#[test]
fn seed_publication_does_not_mutate_a_hard_link_target() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    let directory = TestDirectory::create("fuzz-seed-hard-link")?;
    let root = directory.path();
    let target = root.join("fuzz/corpus/blob_hasher");
    fs::create_dir_all(&target)
        .map_err(|source| FuzzSeedError::io("create test corpus", &target, source))?;
    let protected = root.join("protected");
    fs::write(&protected, b"authoritative")
        .map_err(|source| FuzzSeedError::io("write protected test file", &protected, source))?;
    let destination = target.join("empty");
    fs::hard_link(&protected, &destination)
        .map_err(|source| FuzzSeedError::io("link test seed destination", &destination, source))?;

    RepositoryFiles::open(root)?.write_seeds(&[Seed {
        target: "blob_hasher",
        name: "empty",
        content: b"derived".to_vec(),
    }])?;
    let protected_content = fs::read(&protected)
        .map_err(|source| FuzzSeedError::io("read protected test file", &protected, source))?;
    let seed_content = fs::read(&destination)
        .map_err(|source| FuzzSeedError::io("read test seed", &destination, source))?;
    assert_eq!(protected_content, b"authoritative");
    assert_eq!(seed_content, b"derived");
    directory.close()?;
    Ok(())
}

#[test]
fn interrupted_seed_stage_is_recovered() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    let directory = TestDirectory::create("fuzz-seed-recovery")?;
    let root = directory.path();
    let target = root.join("fuzz/corpus/blob_hasher");
    fs::create_dir_all(&target)
        .map_err(|source| FuzzSeedError::io("create test corpus", &target, source))?;
    let staged = target.join(".empty.keep-tmp");
    fs::write(&staged, b"interrupted")
        .map_err(|source| FuzzSeedError::io("write interrupted seed stage", &staged, source))?;

    RepositoryFiles::open(root)?.write_seeds(&[Seed {
        target: "blob_hasher",
        name: "empty",
        content: b"derived".to_vec(),
    }])?;

    assert_eq!(
        fs::read(target.join("empty"))?,
        b"derived",
        "the recovered publication must contain the requested seed"
    );
    assert!(!staged.exists());
    directory.close()?;
    Ok(())
}

#[test]
fn failed_seed_publication_removes_its_stage() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    let directory = TestDirectory::create("fuzz-seed-failed-publication")?;
    let root = directory.path();
    let target = root.join("fuzz/corpus/blob_hasher");
    fs::create_dir_all(target.join("empty"))
        .map_err(|source| FuzzSeedError::io("create blocking destination", &target, source))?;
    let staged = target.join(".empty.keep-tmp");

    let result = RepositoryFiles::open(root)?.write_seeds(&[Seed {
        target: "blob_hasher",
        name: "empty",
        content: b"derived".to_vec(),
    }]);

    assert!(matches!(
        result,
        Err(FuzzSeedError::Io {
            action: "publish seed",
            ..
        })
    ));
    assert!(!staged.exists());
    directory.close()?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn symlinked_seed_destination_is_refused() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use std::os::unix::fs::symlink;

    let directory = TestDirectory::create("fuzz-seed-destination-link")?;
    let root = directory.path();
    let fuzz = root.join("fuzz");
    let outside = root.join("outside-corpus");
    fs::create_dir(&fuzz)
        .map_err(|source| FuzzSeedError::io("create fuzz test root", &fuzz, source))?;
    fs::create_dir(&outside)
        .map_err(|source| FuzzSeedError::io("create outside corpus", &outside, source))?;
    let corpus = fuzz.join("corpus");
    symlink(&outside, &corpus)
        .map_err(|source| FuzzSeedError::io("link corpus destination", &corpus, source))?;

    let result = RepositoryFiles::open(root)?.write_seeds(&[Seed {
        target: "blob_hasher",
        name: "empty",
        content: Vec::new(),
    }]);

    assert!(matches!(
        result,
        Err(FuzzSeedError::Violation(ref message))
            if message
                == &format!(
                    "seed destination is not a real directory: {}",
                    corpus.display()
                )
    ));
    directory.close()?;
    Ok(())
}
