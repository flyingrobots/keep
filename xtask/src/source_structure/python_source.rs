//! This module owns bounded executable Python-shebang admission.

mod environment;

use std::fs::File;
use std::io::{self, Read};
use std::os::unix::fs::PermissionsExt;

use crate::repository_file::{OpenRepositoryFileError, RepositoryRoot};

use super::SourceStructureError;
use std::path::Path;

const SHEBANG_SCAN_BYTES: u64 = 1_024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FileExecution {
    Executable,
    NonExecutable,
    NonRegular,
}

pub(super) fn refuse_executable_python(
    source_root: &RepositoryRoot,
    relative: &Path,
) -> Result<FileExecution, SourceStructureError> {
    let path = source_root.display_path(relative);
    let file = match source_root.open_file(relative) {
        Ok(file) => file,
        Err(OpenRepositoryFileError::NonRegular) => return Ok(FileExecution::NonRegular),
        Err(OpenRepositoryFileError::Io(source)) => {
            return Err(SourceStructureError::Inspect { path, source });
        }
    };
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
        Err(SourceStructureError::PythonSource(relative.to_owned()))
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
    let line = line.trim_ascii_start();
    let mut fields = line.splitn(2, u8::is_ascii_whitespace);
    let Some(interpreter) = fields.next().filter(|field| !field.is_empty()) else {
        return false;
    };
    if is_python_program(interpreter) {
        return true;
    }
    if !program_name(interpreter).eq_ignore_ascii_case(b"env") {
        return false;
    }
    match fields.next().and_then(environment::selected_utility) {
        Some(environment::UtilitySelection::Known(utility)) => is_python_program(&utility),
        Some(environment::UtilitySelection::Ambiguous) => true,
        None => false,
    }
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
            b"#!/usr/bin/env sh -c python3\n",
            b"#!/usr/bin/env -S sh -c 'echo python3'\n",
            b"#!/usr/bin/env -S \"sh -c 'echo python3'\"\n",
            b"python3\n",
            b"first line\n#!/usr/bin/python3\n",
        ] {
            assert!(!is_python_shebang(prefix));
        }
    }

    #[test]
    fn unresolved_environment_utility_substitution_fails_closed() {
        assert!(is_python_shebang(
            b"#!/usr/bin/env -S '${UNSET_INTERPRETER}sh'\n"
        ));
    }
}
