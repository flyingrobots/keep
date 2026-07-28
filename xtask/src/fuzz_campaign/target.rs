//! This module owns deterministic fuzz target discovery and reconciliation.

mod error;
#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::process::Command;

pub(super) use error::TargetError;

use super::policy::CampaignPolicy;
use super::process;

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct FuzzTarget(String);

impl FuzzTarget {
    pub(super) fn as_str(&self) -> &str {
        &self.0
    }

    pub(super) fn admit(value: String) -> Result<Self, TargetError> {
        let mut bytes = value.bytes();
        let Some(first) = bytes.next() else {
            return Err(TargetError::Malformed(value));
        };
        if !first.is_ascii_lowercase()
            || !bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(TargetError::Malformed(value));
        }
        Ok(Self(value))
    }
}

pub(super) fn registered(
    repository_root: &Path,
    policy: &CampaignPolicy,
) -> Result<Vec<FuzzTarget>, TargetError> {
    let expected = harnesses(repository_root)?;
    let cargo = cargo_executable();
    let mut command = Command::new(&cargo);
    command
        .arg(format!("+{}", policy.toolchain()))
        .args(["fuzz", "list"])
        .current_dir(repository_root);
    let output = process::capture(&mut command, None)?;
    if !output.succeeded {
        return Err(TargetError::ListFailed {
            stdout: output.stdout,
            stderr: output.stderr,
        });
    }
    let observed = parse_list(output.stdout)?;
    if expected == observed {
        Ok(observed)
    } else {
        Err(TargetError::Disagreement {
            expected: names(expected),
            observed: names(observed),
        })
    }
}

pub(super) fn harnesses(repository_root: &Path) -> Result<Vec<FuzzTarget>, TargetError> {
    let directory = repository_root.join("fuzz/fuzz_targets");
    let entries = fs::read_dir(&directory).map_err(|source| TargetError::ReadDirectory {
        path: directory,
        source,
    })?;
    let mut targets = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| TargetError::ReadEntry { source })?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        if !entry
            .file_type()
            .map_err(|source| TargetError::Inspect {
                path: path.clone(),
                source,
            })?
            .is_file()
        {
            return Err(TargetError::NonRegular(path));
        }
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| TargetError::MalformedPath(path.clone()))?;
        targets.push(FuzzTarget::admit(stem.to_owned())?);
    }
    targets.sort();
    if targets.is_empty() {
        Err(TargetError::EmptyHarnesses)
    } else {
        Ok(targets)
    }
}

fn parse_list(bytes: Vec<u8>) -> Result<Vec<FuzzTarget>, TargetError> {
    let text = String::from_utf8(bytes).map_err(|_source| TargetError::InvalidEncoding)?;
    let mut unique = BTreeSet::new();
    for line in text.lines() {
        let target = FuzzTarget::admit(line.trim().to_owned())?;
        if !unique.insert(target) {
            return Err(TargetError::Duplicate);
        }
    }
    if unique.is_empty() {
        Err(TargetError::EmptyRegistry)
    } else {
        Ok(unique.into_iter().collect())
    }
}

fn cargo_executable() -> OsString {
    OsString::from("cargo")
}

fn names(targets: Vec<FuzzTarget>) -> Vec<String> {
    targets.into_iter().map(|target| target.0).collect()
}
