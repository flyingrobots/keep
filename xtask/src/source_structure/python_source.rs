//! This module owns bounded executable Python-shebang admission.

use std::fs::File;
use std::io::{self, Read};
use std::os::unix::fs::PermissionsExt;

use crate::repository_file::{OpenRepositoryFileError, RepositoryRoot};

use super::SourceStructureError;
use super::repository_path::RepositoryPath;

const SHEBANG_SCAN_BYTES: u64 = 1_024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FileExecution {
    Executable,
    NonExecutable,
}

pub(super) fn refuse_executable_python(
    source_root: &RepositoryRoot,
    relative: &RepositoryPath,
) -> Result<FileExecution, SourceStructureError> {
    let path = source_root.display_path(relative.as_path());
    let file = source_root
        .open_file(relative.as_path())
        .map_err(|error| match error {
            OpenRepositoryFileError::Io(source) => SourceStructureError::Inspect {
                path: path.clone(),
                source,
            },
            OpenRepositoryFileError::NonRegular => SourceStructureError::NonRegular(path.clone()),
        })?;
    let execution = file_execution(&file).map_err(|source| SourceStructureError::Inspect {
        path: path.clone(),
        source,
    })?;
    let python = execution == FileExecution::Executable
        && executable_uses_python(file).map_err(|source| SourceStructureError::Inspect {
            path: path.clone(),
            source,
        })?;
    if python {
        Err(SourceStructureError::PythonSource(
            relative.as_str().to_owned(),
        ))
    } else {
        Ok(execution)
    }
}

fn file_execution(file: &File) -> Result<FileExecution, io::Error> {
    if file.metadata()?.permissions().mode() & 0o111 == 0 {
        Ok(FileExecution::NonExecutable)
    } else {
        Ok(FileExecution::Executable)
    }
}

fn executable_uses_python(file: File) -> Result<bool, io::Error> {
    let mut prefix = Vec::new();
    file.take(SHEBANG_SCAN_BYTES).read_to_end(&mut prefix)?;
    Ok(is_python_shebang(&prefix))
}

fn is_python_shebang(prefix: &[u8]) -> bool {
    let Some(line) = prefix
        .split(|byte| *byte == b'\n')
        .next()
        .and_then(|line| line.strip_prefix(b"#!"))
    else {
        return false;
    };
    let mut words = line
        .split(u8::is_ascii_whitespace)
        .filter(|word| !word.is_empty());
    let Some(interpreter) = words.next() else {
        return false;
    };
    if is_python_program(interpreter) {
        return true;
    }
    if !program_name(interpreter).eq_ignore_ascii_case(b"env") {
        return false;
    }
    words.any(environment_word_selects_python)
}

fn environment_word_selects_python(word: &[u8]) -> bool {
    if let Some(split) = word.strip_prefix(b"--split-string=") {
        return is_python_program(split);
    }
    if let Some(split) = word.strip_prefix(b"-S").filter(|split| !split.is_empty()) {
        return is_python_program(split);
    }
    !word.starts_with(b"-") && !word.contains(&b'=') && is_python_program(word)
}

fn is_python_program(program: &[u8]) -> bool {
    let unquoted = program
        .strip_prefix(b"\"")
        .or_else(|| program.strip_prefix(b"'"))
        .unwrap_or(program);
    let name = program_name(unquoted);
    starts_with_ignore_ascii_case(name, b"python") || starts_with_ignore_ascii_case(name, b"pypy")
}

fn program_name(program: &[u8]) -> &[u8] {
    program
        .rsplit(|byte| *byte == b'/')
        .next()
        .unwrap_or(program)
}

fn starts_with_ignore_ascii_case(value: &[u8], prefix: &[u8]) -> bool {
    value
        .get(..prefix.len())
        .is_some_and(|observed| observed.eq_ignore_ascii_case(prefix))
}

#[cfg(test)]
mod tests {
    use super::is_python_shebang;

    #[test]
    fn direct_and_environment_python_interpreters_are_detected() {
        for shebang in [
            b"#!/usr/bin/python3\n".as_slice(),
            b"#! /usr/bin/env python3 -I\n",
            b"#!/usr/bin/env -S python3 -I\n",
            b"#!/usr/bin/env -S \"python3 -I\"\n",
            b"#!/usr/bin/env -Spython3 -I\n",
            b"#!/usr/bin/env -S/opt/PyPy3 -I\n",
            b"#!/usr/bin/env --split-string=python3\n",
            b"#!/opt/PyPy3\n",
        ] {
            assert!(is_python_shebang(shebang));
        }
    }

    #[test]
    fn non_python_or_displaced_interpreters_are_not_misclassified() {
        for prefix in [
            b"#!/bin/sh\n".as_slice(),
            b"#!/usr/bin/env bash\n",
            b"python3\n",
            b"first line\n#!/usr/bin/python3\n",
        ] {
            assert!(!is_python_shebang(prefix));
        }
    }
}
