//! This module owns documentation source-corpus regression evidence.

use std::fs;
use std::path::Path;
use std::process::Command;

use super::{CorpusKind, SourceCorpus, admit_path};
use crate::documentation_integrity::error::DocumentationError;
use crate::git_inventory::GitPath;
use crate::test_directory::TestDirectory;

#[test]
fn markdown_corpus_is_the_sorted_present_repository_set() -> Result<(), Box<dyn std::error::Error>>
{
    let directory = TestDirectory::create("documentation-markdown-corpus")?;
    let root = directory.path();
    run_git(root, &["init", "--quiet"])?;
    write(root, ".gitignore", "/target/\n")?;
    write(root, "zulu.md", "# Zulu\n")?;
    write(root, "alpha.md", "# Alpha\n")?;
    write(root, "deleted.md", "# Deleted\n")?;
    write(root, "target/generated.md", "# Generated\n")?;
    run_git(root, &["add", ".gitignore", "zulu.md", "deleted.md"])?;
    fs::remove_file(root.join("deleted.md"))?;

    let corpus = SourceCorpus::markdown(root)?;

    assert_eq!(corpus.paths(), ["alpha.md", "zulu.md"]);
    directory.close()?;
    Ok(())
}

#[test]
fn workflow_corpus_is_the_sorted_present_repository_set() -> Result<(), Box<dyn std::error::Error>>
{
    let directory = TestDirectory::create("documentation-workflow-corpus")?;
    let root = directory.path();
    run_git(root, &["init", "--quiet"])?;
    write(root, ".gitignore", "/.github/workflows/generated.yml\n")?;
    write(root, ".github/workflows/zulu.yml", "name: Zulu\n")?;
    write(root, ".github/workflows/alpha.yaml", "name: Alpha\n")?;
    write(root, ".github/workflows/deleted.yml", "name: Deleted\n")?;
    write(root, ".github/workflows/generated.yml", "name: Generated\n")?;
    run_git(
        root,
        &[
            "add",
            ".gitignore",
            ".github/workflows/zulu.yml",
            ".github/workflows/deleted.yml",
        ],
    )?;
    fs::remove_file(root.join(".github/workflows/deleted.yml"))?;

    let corpus = SourceCorpus::workflow(root)?;

    assert_eq!(
        corpus.paths(),
        [".github/workflows/alpha.yaml", ".github/workflows/zulu.yml"]
    );
    directory.close()?;
    Ok(())
}

#[test]
fn repository_configured_global_ignores_cannot_change_the_corpus()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("documentation-global-ignore")?;
    let root = directory.path();
    run_git(root, &["init", "--quiet"])?;
    write(root, "tracked.md", "# Tracked\n")?;
    write(root, "new.md", "# New\n")?;
    write(root, "global-ignore", "*.md\n")?;
    run_git(root, &["add", "tracked.md"])?;
    let global_ignore = root.join("global-ignore");
    let global_ignore = global_ignore
        .to_str()
        .ok_or("test path is not valid Unicode")?;
    run_git(root, &["config", "core.excludesFile", global_ignore])?;

    let corpus = SourceCorpus::markdown(root)?;

    assert_eq!(corpus.paths(), ["new.md", "tracked.md"]);
    directory.close()?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn symlinked_markdown_is_refused() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::symlink;

    let directory = TestDirectory::create("documentation-markdown-symlink")?;
    let root = directory.path();
    run_git(root, &["init", "--quiet"])?;
    write(root, "target.txt", "target\n")?;
    symlink("target.txt", root.join("linked.md"))?;

    let result = SourceCorpus::markdown(root);

    assert!(matches!(
        result,
        Err(DocumentationError::NonRegular {
            corpus: "Markdown",
            ref path,
        }) if path == "linked.md"
    ));
    directory.close()?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn fifo_workflow_is_refused() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("documentation-workflow-fifo")?;
    let root = directory.path();
    run_git(root, &["init", "--quiet"])?;
    write(root, ".github/workflows/blocking.yml", "name: Blocking\n")?;
    run_git(root, &["add", ".github/workflows/blocking.yml"])?;
    let fifo = root.join(".github/workflows/blocking.yml");
    fs::remove_file(&fifo)?;
    let status = Command::new("mkfifo").arg(&fifo).status()?;
    if !status.success() {
        return Err("mkfifo fixture command failed".into());
    }

    let result = SourceCorpus::workflow(root);

    assert!(matches!(
        result,
        Err(DocumentationError::NonRegular {
            corpus: "GitHub Actions workflow",
            ref path,
        }) if path == ".github/workflows/blocking.yml"
    ));
    directory.close()?;
    Ok(())
}

#[test]
fn non_utf8_markdown_path_is_refused() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("documentation-markdown-non-utf8")?;
    let root = directory.path();
    let path = GitPath::new(b"bad\xff.md".to_vec());

    let result = admit_path(root, &path, CorpusKind::Markdown);

    assert!(matches!(
        result,
        Err(DocumentationError::PathEncoding {
            corpus: "Markdown",
            ..
        })
    ));
    directory.close()?;
    Ok(())
}

fn run_git(root: &Path, arguments: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(root)
        .output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!("git fixture command failed: {arguments:?}").into())
    }
}

fn write(root: &Path, relative: &str, contents: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
}
