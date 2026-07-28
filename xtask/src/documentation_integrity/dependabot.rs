//! This module owns Dependabot manifest coverage and maintenance policy.

mod manifest;

use std::collections::BTreeSet;
use std::path::Path;

use crate::repository_file::RepositoryRoot;

use super::error::DocumentationError;
use super::repository_text;
use manifest::tracked_scopes;

const DEPENDABOT_PATH: &str = ".github/dependabot.yml";
const UPDATE_MARKER: &str = "  - package-ecosystem: ";

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct DependencyScope {
    ecosystem: String,
    directory: String,
}

pub(super) fn check(
    repository_path: &Path,
    repository_root: &RepositoryRoot,
) -> Result<(), DocumentationError> {
    let raw = repository_text::read(repository_root, DEPENDABOT_PATH)?;
    let required = tracked_scopes(repository_path)?;
    admit(&raw, &required)
}

fn admit(raw: &str, required: &BTreeSet<DependencyScope>) -> Result<(), DocumentationError> {
    if !raw.starts_with("version: 2\nupdates:\n") {
        return Err(contract("version and updates header is exact"));
    }
    let blocks = update_blocks(raw);
    if blocks.is_empty() {
        return Err(contract("at least one update block exists"));
    }
    let mut configured = BTreeSet::new();
    for block in blocks {
        let scopes = block_scopes(&block)?;
        admit_maintenance_policy(&block, &scopes)?;
        for scope in scopes {
            if !configured.insert(scope.clone()) {
                return Err(contract_at(
                    scope.diagnostic(),
                    "update scope appears exactly once",
                ));
            }
        }
    }
    if let Some(missing) = required.difference(&configured).next() {
        return Err(contract_at(
            missing.diagnostic(),
            "tracked dependency scope has an update policy",
        ));
    }
    Ok(())
}

fn update_blocks(raw: &str) -> Vec<Vec<&str>> {
    let mut blocks = Vec::new();
    let mut block = Vec::new();
    let mut active = false;
    for line in raw.lines() {
        if line.starts_with(UPDATE_MARKER) {
            if active {
                blocks.push(std::mem::take(&mut block));
            }
            active = true;
        }
        if active {
            block.push(line);
        }
    }
    if active {
        blocks.push(block);
    }
    blocks
}

fn block_scopes(block: &[&str]) -> Result<Vec<DependencyScope>, DocumentationError> {
    let ecosystem = block
        .first()
        .and_then(|line| line.strip_prefix(UPDATE_MARKER))
        .map(unquote)
        .ok_or_else(|| contract("every update block names an ecosystem"))?;
    let mut scopes = Vec::new();
    let mut lines = block.iter();
    while let Some(line) = lines.next() {
        if let Some(directory) = line.strip_prefix("    directory: ") {
            scopes.push(DependencyScope::new(ecosystem, unquote(directory)));
        } else if *line == "    directories:" {
            scopes.extend(
                lines
                    .by_ref()
                    .map_while(|entry| entry.strip_prefix("      - "))
                    .map(|directory| DependencyScope::new(ecosystem, unquote(directory))),
            );
        }
    }
    if scopes.is_empty() {
        Err(contract_at(
            ecosystem.to_owned(),
            "update block names at least one directory",
        ))
    } else {
        Ok(scopes)
    }
}

fn admit_maintenance_policy(
    block: &[&str],
    scopes: &[DependencyScope],
) -> Result<(), DocumentationError> {
    let raw = block.join("\n");
    let uniform = raw.contains("    schedule:\n      interval: weekly")
        && raw.contains("    open-pull-requests-limit: 5")
        && raw.contains("    labels:\n      - dependencies");
    if uniform {
        Ok(())
    } else {
        let subject = scopes
            .first()
            .map_or_else(String::new, DependencyScope::diagnostic);
        Err(contract_at(
            subject,
            "update block uses the maintenance policy",
        ))
    }
}

fn unquote(raw: &str) -> &str {
    let bytes = raw.as_bytes();
    match (bytes.first(), bytes.last()) {
        (Some(first), Some(last))
            if bytes.len() >= 2 && first == last && matches!(first, b'\'' | b'"') =>
        {
            raw.get(1..raw.len().saturating_sub(1)).unwrap_or(raw)
        }
        _ => raw,
    }
}

impl DependencyScope {
    fn new(ecosystem: &str, directory: &str) -> Self {
        Self {
            ecosystem: ecosystem.to_owned(),
            directory: directory.to_owned(),
        }
    }

    fn diagnostic(&self) -> String {
        format!("{} {}", self.ecosystem, self.directory)
    }
}

const fn contract(requirement: &'static str) -> DocumentationError {
    DocumentationError::RepositoryContract {
        path: DEPENDABOT_PATH,
        requirement,
    }
}

const fn contract_at(subject: String, requirement: &'static str) -> DocumentationError {
    DocumentationError::RepositoryContractAt {
        path: DEPENDABOT_PATH,
        subject,
        requirement,
    }
}

#[cfg(test)]
#[path = "dependabot/tests.rs"]
mod tests;
