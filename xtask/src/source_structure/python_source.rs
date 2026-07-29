//! This module owns bounded executable Python-shebang admission.

mod environment;

use std::fs::File;
use std::io;
use std::os::unix::fs::FileExt;

const SHEBANG_SCAN_BYTES: usize = 1_024;

pub(super) fn executable_uses_python(file: &File) -> Result<bool, io::Error> {
    let mut prefix = [0_u8; SHEBANG_SCAN_BYTES];
    let bytes = read_prefix(file, &mut prefix)?;
    let admitted = prefix.get(..bytes).ok_or_else(prefix_bounds_error)?;
    Ok(is_python_shebang(admitted))
}

fn read_prefix(file: &File, prefix: &mut [u8]) -> Result<usize, io::Error> {
    let mut filled = 0_usize;
    while filled < prefix.len() {
        let offset = u64::try_from(filled).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "shebang prefix offset exceeds u64",
            )
        })?;
        let remaining = prefix.get_mut(filled..).ok_or_else(prefix_bounds_error)?;
        let read = match file.read_at(remaining, offset) {
            Err(source) if source.kind() == io::ErrorKind::Interrupted => continue,
            result => result?,
        };
        if read == 0 {
            break;
        }
        filled = filled.checked_add(read).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "shebang prefix length overflow")
        })?;
    }
    Ok(filled)
}

fn prefix_bounds_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "shebang prefix bounds are inconsistent",
    )
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

    #[test]
    fn combined_environment_options_cannot_hide_python() {
        assert!(is_python_shebang(b"#!/usr/bin/env -S-iuFOO python3\n"));
    }
}
