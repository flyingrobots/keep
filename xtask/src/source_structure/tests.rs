//! This module owns source-line, selection, and replacement-race tests.

use std::io::{self, BufReader, Cursor, Read};
use std::path::Path;

use super::{PRESENT_PATH_ARGUMENTS, exceeds_hard_limit, is_source_module, line_count};

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
fn source_read_options_refuse_blocking_io() {
    let options = format!("{:?}", super::source_file::nonblocking_read_options());
    assert!(options.contains("read: true"));
    assert!(options.contains("nonblock: true"));
}

#[cfg(unix)]
#[test]
fn source_open_refuses_replacement_symlink() -> Result<(), super::SourceStructureError> {
    use std::env;
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::process;

    use super::{open_source_file, source_line_count_with};

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

    let result = source_line_count_with(&root, "source.rs", |repository_root, relative| {
        let admitted = repository_root.join(relative);
        fs::rename(&admitted, &retained_path).map_err(super::OpenSourceError::Io)?;
        symlink(&target_path, &admitted).map_err(super::OpenSourceError::Io)?;
        open_source_file(repository_root, relative)
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
    assert!(is_source_module(Path::new("src/lib.rs")));
    assert!(is_source_module(Path::new("scripts/check.py")));
    assert!(is_source_module(Path::new("scripts/check.sh")));
    assert!(!is_source_module(Path::new("README.md")));
    assert!(!is_source_module(Path::new("src/lib.RS")));
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
