//! This module owns documentation source-replacement regression evidence.

use std::fs;

use super::SourceCorpus;
use crate::documentation_integrity::error::DocumentationError;
use crate::repository_file::RepositoryRoot;
use crate::repository_fixture::run_git;
use crate::test_directory::TestDirectory;

#[test]
fn selected_source_replacement_refuses_the_admitted_corpus()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("documentation-source-replacement")?;
    let root = directory.path();
    run_git(root, &["init", "--quiet", "--template="])?;
    fs::write(root.join("selected.md"), "# Original\n")?;
    let repository_root = RepositoryRoot::open(root)?;
    let process_directory = repository_root.process_directory()?;
    let corpus = SourceCorpus::markdown(&repository_root, &process_directory)?;

    fs::rename(root.join("selected.md"), root.join("retained.md"))?;
    fs::write(root.join("selected.md"), "# Substitute\n")?;

    let result = corpus.verify_unchanged(&repository_root, &process_directory);

    assert!(matches!(
        result,
        Err(DocumentationError::CorpusChanged {
            corpus: "Markdown",
            ref path,
        }) if path == "selected.md"
    ));
    drop(corpus);
    directory.close()?;
    Ok(())
}

#[test]
fn added_source_refuses_the_admitted_corpus() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("documentation-source-addition")?;
    let root = directory.path();
    run_git(root, &["init", "--quiet", "--template="])?;
    fs::write(root.join("selected.md"), "# Original\n")?;
    let repository_root = RepositoryRoot::open(root)?;
    let process_directory = repository_root.process_directory()?;
    let corpus = SourceCorpus::markdown(&repository_root, &process_directory)?;

    fs::write(root.join("added.md"), "# Added\n")?;

    let result = corpus.verify_unchanged(&repository_root, &process_directory);

    assert!(matches!(
        result,
        Err(DocumentationError::CorpusChanged {
            corpus: "Markdown",
            ref path,
        }) if path == "added.md"
    ));
    drop(corpus);
    directory.close()?;
    Ok(())
}

#[test]
fn source_replaced_between_identity_and_membership_checks_is_refused()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("documentation-source-verification-race")?;
    let root = directory.path();
    run_git(root, &["init", "--quiet", "--template="])?;
    fs::write(root.join("selected.md"), "# Original\n")?;
    fs::write(root.join("substitute.bin"), "# Substitute\n")?;
    let repository_root = RepositoryRoot::open(root)?;
    let process_directory = repository_root.process_directory()?;
    let corpus = SourceCorpus::markdown(&repository_root, &process_directory)?;

    let result = corpus.verify_unchanged_with(&repository_root, &process_directory, || {
        fs::rename(root.join("substitute.bin"), root.join("selected.md")).map_err(|source| {
            DocumentationError::Snapshot {
                action: "replace source between corpus verification phases",
                source,
            }
        })
    });

    assert!(matches!(
        result,
        Err(DocumentationError::CorpusChanged {
            corpus: "Markdown",
            ref path,
        }) if path == "selected.md"
    ));
    drop(corpus);
    directory.close()?;
    Ok(())
}
