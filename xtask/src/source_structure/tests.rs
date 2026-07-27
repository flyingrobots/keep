//! This module owns source-line, selection, and replacement-race tests.

use super::{PRESENT_PATH_ARGUMENTS, exceeds_hard_limit, is_source_module, line_count};
use std::io::{self, BufReader, Cursor, Read};

#[test]
fn line_count_observes_empty_and_final_newline_edges() {
    assert_eq!(line_count(Cursor::new(b"")).ok(), Some(0));
    assert_eq!(line_count(Cursor::new(b"a")).ok(), Some(1));
    assert_eq!(line_count(Cursor::new(b"a\n")).ok(), Some(1));
    assert_eq!(line_count(Cursor::new(b"a\nb")).ok(), Some(2));
    assert_eq!(line_count(Cursor::new(b"a\nb\n")).ok(), Some(2));
}

#[test]
fn source_scan_stops_at_the_first_violating_line() {
    let lines = "x\n".repeat(501);
    let reader = Cursor::new(lines.into_bytes()).chain(RefuseTail);
    assert_eq!(line_count(BufReader::new(reader)).ok(), Some(501));
}

#[test]
fn source_structure_diagnostics_are_stable() {
    let framing = super::SourceStructureError::GitOutputFraming {
        operation: "git inventory",
    };
    let non_regular = super::SourceStructureError::NonRegular("src/link.rs".into());
    assert_eq!(
        framing.to_string(),
        "`git inventory` returned a non-NUL-terminated path"
    );
    assert_eq!(
        non_regular.to_string(),
        "tracked source module is not a regular file: `src/link.rs`"
    );
    let violations = super::SourceStructureError::Violations {
        maximum: 7,
        paths: vec![(String::from("src/large.rs"), 8)],
    };
    assert_eq!(
        violations.to_string(),
        "tracked source modules exceed the 7-line hard maximum; src/large.rs: 8"
    );
}

#[test]
fn git_diagnostics_cannot_inject_terminal_control_lines() {
    let error = super::SourceStructureError::GitFailed {
        operation: "git inventory",
        code: Some(9),
        stderr: String::from("first\nError: forged\rrewrite\u{1b}[31m"),
    };
    let diagnostic = error.to_string();
    assert_eq!(
        diagnostic,
        "`git inventory` failed with code Some(9): \
         first\\nError: forged\\rrewrite\\u{1b}[31m"
    );
    assert_eq!(diagnostic.lines().count(), 1);
}

#[test]
fn source_read_options_refuse_blocking_io() {
    let options = format!("{:?}", super::source_file::nonblocking_read_options());
    assert!(options.contains("read: true"));
    assert!(options.contains("nonblock: true"));
}

#[cfg(unix)]
#[test]
fn source_scan_keeps_the_admitted_repository_root() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use std::fs;
    use std::process;

    use super::repository_path::RepositoryPath;
    use super::source_file::SourceRoot;
    use super::source_line_count;

    let root = env::temp_dir().join(format!("keep-source-root-{}", process::id()));
    let retained_root = root.with_extension("retained");
    fs::create_dir(&root)?;
    fs::write(root.join("source.rs"), "safe\n")?;
    let source_root = SourceRoot::open(&root)?;
    let relative = RepositoryPath::admit(String::from("source.rs"))?;

    fs::rename(&root, &retained_root)?;
    fs::create_dir(&root)?;
    fs::write(root.join("source.rs"), "replacement\n".repeat(501))?;

    let line_count = source_line_count(&source_root, &relative)?;
    fs::remove_dir_all(&root)?;
    fs::remove_dir_all(&retained_root)?;

    assert_eq!(line_count, 1);
    Ok(())
}

#[cfg(unix)]
#[test]
fn source_open_refuses_replacement_symlink() -> Result<(), super::SourceStructureError> {
    use std::env;
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::process;

    use super::repository_path::RepositoryPath;
    use super::source_file::SourceRoot;
    use super::source_line_count_with;

    let root = env::temp_dir().join(format!("keep-source-replacement-{}", process::id()));
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
        SourceRoot::open(&root).map_err(|source| super::SourceStructureError::Inspect {
            path: root.clone(),
            source,
        })?;
    let relative = RepositoryPath::admit(String::from("source.rs"))?;

    let result = source_line_count_with(&source_root, &relative, |source_root, relative| {
        let admitted = source_root.display_path(relative);
        fs::rename(&admitted, &retained_path).map_err(super::OpenSourceError::Io)?;
        symlink(&target_path, &admitted).map_err(super::OpenSourceError::Io)?;
        source_root.open_file(relative)
    });
    let refused = matches!(
        result,
        Err(super::SourceStructureError::Inspect {
            ref path,
            source: _,
        }) if path == &source_path
    );
    fs::remove_dir_all(&root).map_err(|source| super::SourceStructureError::Inspect {
        path: root.clone(),
        source,
    })?;

    assert!(refused);
    Ok(())
}

#[test]
fn source_module_limit_accepts_five_hundred_and_refuses_five_hundred_one() {
    assert!(!exceeds_hard_limit(500));
    assert!(exceeds_hard_limit(501));
}

#[test]
fn source_module_classification_is_explicit() {
    assert!(is_source_module("src/lib.rs"));
    assert!(is_source_module("scripts/check.py"));
    assert!(is_source_module("scripts/check.sh"));
    assert!(!is_source_module("README.md"));
    assert!(!is_source_module("src/lib.RS"));
    assert!(!is_source_module(".rs"));
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
