//! This module owns deterministic fuzz seed recipes and materialization.

mod cdc_seeds;
mod filesystem;
mod identity_seeds;

use std::error::Error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use crate::diagnostic::{escaped_controls, escaped_path};
use filesystem::RepositoryFiles;

const MAX_SEED_BYTES: usize = 1_048_576;
const GOLDEN_ROOT: &str = "conformance/golden-file-worldline/v1";
const TABLE_SEEDS: [(u8, &str, &str); 5] = [
    (0, "identities-table", "identities.tsv"),
    (1, "invalid-text-table", "invalid-text.tsv"),
    (2, "mutations-table", "mutations.tsv"),
    (3, "steps-table", "steps.tsv"),
    (4, "capabilities-table", "capabilities.tsv"),
];

pub(super) enum FuzzSeedError {
    Io {
        action: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    #[cfg(test)]
    RepositoryRoot,
    Violation(String),
}

struct Seed {
    target: &'static str,
    name: &'static str,
    content: Vec<u8>,
}

impl Seed {
    fn new(
        target: &'static str,
        name: &'static str,
        content: Vec<u8>,
    ) -> Result<Self, FuzzSeedError> {
        if content.len() <= MAX_SEED_BYTES {
            Ok(Self {
                target,
                name,
                content,
            })
        } else {
            Err(FuzzSeedError::violation(format!(
                "{target}/{name} exceeds the input bound"
            )))
        }
    }
}

pub(super) fn prepare(repository_root: &Path) -> Result<(), FuzzSeedError> {
    let files = RepositoryFiles::open(repository_root)?;
    let mut seeds = identity_seeds::seeds(&files)?;
    seeds.extend(cdc_seeds::seeds()?);
    seeds.extend(golden_protocol_seeds_from(&files)?);
    files.write_seeds(&seeds)
}

#[cfg(test)]
fn golden_protocol_seeds(repository_root: &Path) -> Result<Vec<Seed>, FuzzSeedError> {
    golden_protocol_seeds_from(&RepositoryFiles::open(repository_root)?)
}

fn golden_protocol_seeds_from(files: &RepositoryFiles) -> Result<Vec<Seed>, FuzzSeedError> {
    let mut seeds = Vec::new();
    for (selector, name, table) in TABLE_SEEDS {
        let relative = Path::new(GOLDEN_ROOT).join(table);
        let content = files.read_bounded(&relative, MAX_SEED_BYTES.saturating_sub(1))?;
        seeds.push(Seed::new(
            "golden_protocol",
            name,
            prefixed(selector, &content)?,
        )?);
    }
    for (selector, name, content) in [
        (5, "canonical-case", b"canonical-case".as_slice()),
        (6, "maximum-decimal", b"18446744073709551615".as_slice()),
        (7, "invalid-identity", b"not-an-identity".as_slice()),
        (8, "xor-byte-mutation", b"00\txor-byte\t0\t01".as_slice()),
    ] {
        seeds.push(Seed::new(
            "golden_protocol",
            name,
            prefixed(selector, content)?,
        )?);
    }
    Ok(seeds)
}

fn prefixed(selector: u8, content: &[u8]) -> Result<Vec<u8>, FuzzSeedError> {
    let capacity = content
        .len()
        .checked_add(1)
        .ok_or_else(|| FuzzSeedError::violation("golden protocol seed length overflow"))?;
    let mut prefixed = Vec::with_capacity(capacity);
    prefixed.push(selector);
    prefixed.extend_from_slice(content);
    Ok(prefixed)
}

impl FuzzSeedError {
    fn io(action: &'static str, path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            action,
            path: path.into(),
            source,
        }
    }

    fn violation(message: impl Into<String>) -> Self {
        Self::Violation(message.into())
    }
}

impl fmt::Debug for FuzzSeedError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for FuzzSeedError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("fuzz seed preparation failed: ")?;
        match self {
            Self::Io { action, path, .. } => {
                write!(formatter, "cannot {action} `")?;
                escaped_path(formatter, path)?;
                formatter.write_str("`")
            }
            #[cfg(test)]
            Self::RepositoryRoot => formatter.write_str("xtask manifest has no repository parent"),
            Self::Violation(message) => escaped_controls(formatter, message),
        }
    }
}

impl Error for FuzzSeedError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            #[cfg(test)]
            Self::RepositoryRoot => None,
            Self::Violation(_) => None,
        }
    }
}

#[cfg(test)]
mod tests;
