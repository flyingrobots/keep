//! This module owns source-line, selection, and replacement-race tests.

use super::source_kind::is_source_module;
use super::{PRESENT_PATH_ARGUMENTS, SourceLineCount, exceeds_hard_limit, line_count};
use crate::test_directory::TestDirectory;
use std::io::{self, BufReader, Cursor, Read};

#[test]
fn line_count_observes_empty_and_final_newline_edges() {
    assert_eq!(
        line_count(Cursor::new(b"")).ok(),
        Some(SourceLineCount::Within(0))
    );
    assert_eq!(
        line_count(Cursor::new(b"a")).ok(),
        Some(SourceLineCount::Within(1))
    );
    assert_eq!(
        line_count(Cursor::new(b"a\n")).ok(),
        Some(SourceLineCount::Within(1))
    );
    assert_eq!(
        line_count(Cursor::new(b"a\nb")).ok(),
        Some(SourceLineCount::Within(2))
    );
    assert_eq!(
        line_count(Cursor::new(b"a\nb\n")).ok(),
        Some(SourceLineCount::Within(2))
    );
}

#[test]
fn source_scan_stops_at_the_first_violating_line() {
    let lines = "x\n".repeat(501);
    let reader = Cursor::new(lines.into_bytes()).chain(RefuseTail);
    assert_eq!(
        line_count(BufReader::new(reader)).ok(),
        Some(SourceLineCount::Exceeded)
    );
}

#[test]
fn early_source_refusal_does_not_claim_an_exact_line_count() {
    let lines = "x\n".repeat(501);
    let reader = Cursor::new(lines.into_bytes()).chain(RefuseTail);
    let diagnostic = line_count(BufReader::new(reader)).map(|observed| match observed {
        SourceLineCount::Exceeded => super::SourceStructureError::Violations {
            maximum: 500,
            paths: vec![std::path::PathBuf::from("src/large.rs")],
        }
        .to_string(),
        SourceLineCount::Within(lines) => format!("unexpected exact count: {lines}"),
    });
    assert_eq!(
        diagnostic.ok().as_deref(),
        Some(
            "repository source modules exceed the 500-line hard maximum; \
             src/large.rs: >500"
        )
    );
}

#[test]
fn source_structure_diagnostics_are_stable() {
    use crate::git_inventory::{GitInventoryError, GitOutputUnit};

    let framing = super::SourceStructureError::GitInventory(GitInventoryError::OutputFraming {
        operation: "git inventory",
    });
    let non_regular = super::SourceStructureError::NonRegular("src/link.rs".into());
    assert_eq!(
        framing.to_string(),
        "`git inventory` returned a non-NUL-terminated path"
    );
    assert_eq!(
        non_regular.to_string(),
        "repository source module is not a regular file: `src/link.rs`"
    );
    let violations = super::SourceStructureError::Violations {
        maximum: 7,
        paths: vec![std::path::PathBuf::from("src/large.rs")],
    };
    assert_eq!(
        violations.to_string(),
        "repository source modules exceed the 7-line hard maximum; src/large.rs: >7"
    );
    let byte_bound = super::SourceStructureError::GitInventory(GitInventoryError::OutputBound {
        operation: "git inventory",
        stream: "path bytes",
        maximum: 4_096,
        unit: GitOutputUnit::Bytes,
    });
    let item_bound = super::SourceStructureError::GitInventory(GitInventoryError::OutputBound {
        operation: "git inventory",
        stream: "path count",
        maximum: 100_000,
        unit: GitOutputUnit::Items,
    });
    assert_eq!(
        byte_bound.to_string(),
        "`git inventory` exceeded the path bytes bound of 4096 bytes"
    );
    assert_eq!(
        item_bound.to_string(),
        "`git inventory` exceeded the path count bound of 100000 items"
    );
}

#[test]
fn git_diagnostics_cannot_inject_terminal_control_lines() {
    use crate::git_inventory::GitInventoryError;

    let error = super::SourceStructureError::GitInventory(GitInventoryError::Failed {
        operation: "git inventory",
        code: Some(9),
        stderr: String::from("first\nError: forged\rrewrite\u{1b}[31m"),
    });
    let diagnostic = error.to_string();
    assert_eq!(
        diagnostic,
        "`git inventory` failed with code Some(9): \
         first\\nError: forged\\rrewrite\\u{1b}[31m"
    );
    assert_eq!(diagnostic.lines().count(), 1);

    let path = super::SourceStructureError::InvalidPath(String::from(
        "first\nError: forged\rrewrite\u{1b}",
    ))
    .to_string();
    assert_eq!(
        path,
        "git returned unsafe path `first\\nError: forged\\rrewrite\\u{1b}`"
    );
    assert_eq!(path.lines().count(), 1);
}

#[test]
fn source_read_policy_enables_reads_and_refuses_blocking_io() {
    use crate::repository_file::{BlockingIoPolicy, REPOSITORY_READ_POLICY, ReadAccessPolicy};

    assert_eq!(
        REPOSITORY_READ_POLICY.read_access(),
        ReadAccessPolicy::Enabled
    );
    assert_eq!(
        REPOSITORY_READ_POLICY.blocking_io(),
        BlockingIoPolicy::Refuse
    );
}

#[cfg(unix)]
#[test]
fn source_scan_keeps_the_admitted_repository_root() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    use crate::repository_file::RepositoryRoot;

    use super::repository_path::RepositoryPath;
    use super::source_line_count;

    let directory = TestDirectory::create("source-root")?;
    let root = directory.path().join("repository");
    let retained_root = directory.path().join("retained");
    fs::create_dir(&root)?;
    fs::write(root.join("source.rs"), "safe\n")?;
    let source_root = RepositoryRoot::open(&root)?;
    let relative = RepositoryPath::admit(String::from("source.rs"))?;

    fs::rename(&root, &retained_root)?;
    fs::create_dir(&root)?;
    fs::write(root.join("source.rs"), "replacement\n".repeat(501))?;

    let line_count = source_line_count(&source_root, relative.as_path())?;
    assert_eq!(line_count, SourceLineCount::Within(1));
    drop(source_root);
    directory.close()?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn source_scan_detects_a_replaced_repository_root() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    use crate::repository_file::RepositoryRoot;

    let directory = TestDirectory::create("source-identity")?;
    let root = directory.path().join("repository");
    let retained_root = directory.path().join("retained");
    fs::create_dir(&root)?;
    let source_root = RepositoryRoot::open(&root)?;

    fs::rename(&root, &retained_root)?;
    fs::create_dir(&root)?;
    let result = super::verify_source_root(&source_root, &root);
    assert!(matches!(
        result,
        Err(super::SourceStructureError::RepositoryRootChanged(ref path)) if path == &root
    ));
    drop(source_root);
    directory.close()?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn source_open_refuses_replacement_symlink() -> Result<(), super::SourceStructureError> {
    use std::fs;
    use std::os::unix::fs::symlink;

    use crate::repository_file::{OpenRepositoryFileError, RepositoryRoot};

    use super::repository_path::RepositoryPath;
    use super::source_line_count_with;

    let directory = TestDirectory::create("source-replacement").map_err(|source| {
        super::SourceStructureError::Inspect {
            path: "scoped test directory".into(),
            source,
        }
    })?;
    let root = directory.path().join("repository");
    fs::create_dir(&root).map_err(|source| super::SourceStructureError::Inspect {
        path: root.clone(),
        source,
    })?;
    let source_path = root.join("source.rs");
    let retained_path = root.join("retained.rs");
    let target_path = root.join("target.rs");
    fs::write(&source_path, "safe\n").map_err(|source| super::SourceStructureError::Inspect {
        path: source_path.clone(),
        source,
    })?;
    fs::write(&target_path, "outside\n".repeat(501)).map_err(|source| {
        super::SourceStructureError::Inspect {
            path: target_path.clone(),
            source,
        }
    })?;
    let source_root =
        RepositoryRoot::open(&root).map_err(|source| super::SourceStructureError::Inspect {
            path: root.clone(),
            source,
        })?;
    let relative = RepositoryPath::admit(String::from("source.rs"))?;

    let result =
        source_line_count_with(&source_root, relative.as_path(), |source_root, relative| {
            let admitted = source_root.display_path(relative);
            fs::rename(&admitted, &retained_path).map_err(OpenRepositoryFileError::Io)?;
            symlink(&target_path, &admitted).map_err(OpenRepositoryFileError::Io)?;
            source_root.open_file(relative)
        });
    let refused = matches!(
        result,
        Err(super::SourceStructureError::NonRegular(ref path)) if path == &source_path
    );
    assert!(refused);
    drop(source_root);
    directory
        .close()
        .map_err(|source| super::SourceStructureError::Inspect { path: root, source })?;
    Ok(())
}

#[test]
fn source_module_limit_accepts_five_hundred_and_refuses_five_hundred_one() {
    assert!(!exceeds_hard_limit(500));
    assert!(exceeds_hard_limit(501));
}

#[test]
fn source_module_classification_is_explicit() {
    assert!(is_source_module(b"src/lib.rs"));
    assert!(is_source_module(b".py"));
    assert!(is_source_module(b"scripts/.PYW"));
    assert!(is_source_module(b"scripts/check.py"));
    assert!(is_source_module(b"scripts/check.pyw"));
    assert!(is_source_module(b"scripts/check.sh"));
    assert!(!is_source_module(b"README.md"));
    assert!(!is_source_module(b"src/lib.RS"));
    assert!(!is_source_module(b".rs"));
}

#[test]
fn source_paths_refuse_host_dependent_spellings() {
    for path in [r"src\host.rs", "C:/drive.rs", "src//empty.rs"] {
        let result = super::repository_path::RepositoryPath::admit(path.to_owned());
        assert!(matches!(
            result,
            Err(super::SourceStructureError::InvalidPath(ref observed)) if observed == path
        ));
    }
}

#[test]
fn source_selection_refuses_an_unsafe_source_record() {
    use std::collections::BTreeSet;

    use crate::git_inventory::GitPath;

    let present = BTreeSet::from([
        GitPath::new(b"../escape.rs".to_vec()),
        GitPath::new(b"notes/x:y".to_vec()),
    ]);
    let result = super::select_source_paths(&present, &BTreeSet::new());
    assert!(matches!(
        result,
        Err(super::SourceStructureError::InvalidPath(ref path)) if path == "../escape.rs"
    ));
}

#[test]
fn source_selection_refuses_a_non_utf8_source_record() {
    use std::collections::BTreeSet;

    use crate::git_inventory::GitPath;

    let present = BTreeSet::from([GitPath::new(b"bad\xff.rs".to_vec())]);
    let result = super::select_source_paths(&present, &BTreeSet::new());
    assert!(matches!(
        result,
        Err(super::SourceStructureError::GitPathEncoding {
            operation: "source path admission",
            ..
        })
    ));
}

#[test]
fn source_selection_ignores_only_repository_owned_patterns() {
    assert_eq!(
        PRESENT_PATH_ARGUMENTS,
        [
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-per-directory=.gitignore",
        ]
    );
}

struct RefuseTail;

impl Read for RefuseTail {
    fn read(&mut self, _buffer: &mut [u8]) -> Result<usize, io::Error> {
        Err(io::Error::other("reader tail must remain untouched"))
    }
}
