//! This module owns documentation snapshot namespace regression evidence.

use std::error::Error;
use std::fs;
use std::io::{self, Read};
use std::os::unix::fs::symlink;
use std::path::Path;

use crate::documentation_integrity::corpus::SourceCorpus;
use crate::repository_file::RepositoryRoot;
use crate::repository_fixture::run_git;
use crate::test_directory::TestDirectory;

use super::DocumentationSnapshot;

#[test]
fn non_markdown_link_target_preserves_admitted_bytes() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::create("documentation-link-target")?;
    let root = directory.path();
    run_git(root, &["init", "--quiet", "--template="])?;
    fs::create_dir(root.join("docs"))?;
    fs::write(
        root.join("docs/README.md"),
        "[target](../target.html#section)\n",
    )?;
    fs::write(root.join("target.html"), "<h2 id=\"section\">Target</h2>\n")?;
    fs::write(
        root.join(".markdownlint-cli2.yaml"),
        "config:\n  MD013: false\n",
    )?;
    run_git(
        root,
        &[
            "add",
            "--",
            ".markdownlint-cli2.yaml",
            "docs/README.md",
            "target.html",
        ],
    )?;
    let repository_root = RepositoryRoot::open(root)?;
    let process_directory = repository_root.process_directory()?;
    let markdown = SourceCorpus::markdown(&repository_root, &process_directory)?;
    let snapshot =
        DocumentationSnapshot::create(&repository_root, &process_directory, &[&markdown])?;
    let mut target = snapshot
        .repository_root()
        .open_file(Path::new("target.html"))
        .map_err(|_| io::Error::other("open snapshot target"))?;
    let mut observed = Vec::new();
    target.read_to_end(&mut observed)?;

    assert_eq!(observed, b"<h2 id=\"section\">Target</h2>\n");
    drop(target);
    snapshot.close()?;
    drop(markdown);
    directory.close()?;
    Ok(())
}

#[test]
fn nonregular_link_target_is_refused_before_validation() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::create("documentation-symlink-target")?;
    let root = directory.path();
    run_git(root, &["init", "--quiet", "--template="])?;
    fs::create_dir(root.join("docs"))?;
    fs::write(root.join("docs/README.md"), "[target](../target.html)\n")?;
    fs::write(root.join("real.html"), "<p>Target</p>\n")?;
    symlink("real.html", root.join("target.html"))?;
    fs::write(
        root.join(".markdownlint-cli2.yaml"),
        "config:\n  MD013: false\n",
    )?;
    run_git(
        root,
        &[
            "add",
            "--",
            ".markdownlint-cli2.yaml",
            "docs/README.md",
            "real.html",
            "target.html",
        ],
    )?;
    let repository_root = RepositoryRoot::open(root)?;
    let process_directory = repository_root.process_directory()?;
    let markdown = SourceCorpus::markdown(&repository_root, &process_directory)?;

    let result = DocumentationSnapshot::create(&repository_root, &process_directory, &[&markdown]);

    assert!(matches!(
        result,
        Err(crate::documentation_integrity::DocumentationError::NonRegular {
            corpus: "documentation snapshot namespace",
            ref path,
        }) if path == "target.html"
    ));
    drop(markdown);
    directory.close()?;
    Ok(())
}
