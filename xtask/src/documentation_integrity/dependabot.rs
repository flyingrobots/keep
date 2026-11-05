//! This module owns Dependabot manifest coverage and maintenance policy.

mod manifest;
mod workspace_members;

use std::collections::BTreeSet;

use yaml_rust2::{Yaml, YamlLoader};

use crate::repository_file::{RepositoryProcessDirectory, RepositoryRoot};

use super::error::DocumentationError;
use super::repository_text::{self, RepositoryText};
use manifest::tracked_scopes;
use workspace_members::ROOT_MANIFEST_PATH;

const DEPENDABOT_PATH: &str = ".github/dependabot.yml";

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct DependencyScope {
    ecosystem: String,
    directory: String,
}

/// Separates scopes Dependabot must update from workspace members it must not name.
/// Dependabot refuses to update from inside a member, and the job for the workspace
/// root already updates member manifests.
struct ScopeContract {
    required: BTreeSet<DependencyScope>,
    members: BTreeSet<DependencyScope>,
}

impl ScopeContract {
    fn new(tracked: &BTreeSet<DependencyScope>, members: BTreeSet<DependencyScope>) -> Self {
        let required = tracked.difference(&members).cloned().collect();
        Self { required, members }
    }
}

pub(super) fn check(
    repository_root: &RepositoryRoot,
    process_directory: &RepositoryProcessDirectory,
) -> Result<RepositoryText, DocumentationError> {
    let raw = repository_text::read(repository_root, DEPENDABOT_PATH)?;
    let manifest = repository_text::read(repository_root, ROOT_MANIFEST_PATH)?;
    let members = workspace_members::admit(manifest.as_str())?;
    let scopes = ScopeContract::new(&tracked_scopes(process_directory)?, members);
    admit(raw.as_str(), &scopes)?;
    manifest.verify(repository_root)?;
    raw.verify(repository_root)?;
    Ok(raw)
}

fn admit(raw: &str, scopes: &ScopeContract) -> Result<(), DocumentationError> {
    let documents =
        YamlLoader::load_from_str(raw).map_err(|source| DocumentationError::RepositoryYaml {
            path: DEPENDABOT_PATH,
            source,
        })?;
    let [document] = documents.as_slice() else {
        return Err(contract("policy contains exactly one YAML document"));
    };
    if document["version"].as_i64() != Some(2) {
        return Err(contract("policy version is exactly 2"));
    }
    let Some(updates) = document["updates"].as_vec() else {
        return Err(contract("updates is a sequence"));
    };
    if updates.is_empty() {
        return Err(contract("at least one update block exists"));
    }
    let mut configured = BTreeSet::new();
    for update in updates {
        let scopes = block_scopes(update)?;
        admit_maintenance_policy(update, &scopes)?;
        for scope in scopes {
            if !configured.insert(scope.clone()) {
                return Err(contract_at(
                    scope.diagnostic(),
                    "update scope appears exactly once",
                ));
            }
        }
    }
    if let Some(member) = scopes.members.intersection(&configured).next() {
        return Err(contract_at(
            member.diagnostic(),
            "workspace member is updated through its workspace root",
        ));
    }
    if let Some(missing) = scopes.required.difference(&configured).next() {
        return Err(contract_at(
            missing.diagnostic(),
            "tracked dependency scope has an update policy",
        ));
    }
    Ok(())
}

fn block_scopes(update: &Yaml) -> Result<Vec<DependencyScope>, DocumentationError> {
    let ecosystem = update["package-ecosystem"]
        .as_str()
        .ok_or_else(|| contract("every update block names an ecosystem"))?;
    let has_directory = !update["directory"].is_badvalue();
    let has_directories = !update["directories"].is_badvalue();
    if has_directory && has_directories {
        return Err(contract("update block chooses one directory form"));
    }
    let mut scopes = Vec::new();
    if has_directory {
        let directory = update["directory"]
            .as_str()
            .ok_or_else(|| contract("update directory is a string"))?;
        scopes.push(DependencyScope::new(ecosystem, directory));
    }
    if has_directories {
        let directories = update["directories"]
            .as_vec()
            .ok_or_else(|| contract("update directories is a sequence"))?;
        for directory in directories {
            let directory = directory
                .as_str()
                .ok_or_else(|| contract("every update directory is a string"))?;
            scopes.push(DependencyScope::new(ecosystem, directory));
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
    update: &Yaml,
    scopes: &[DependencyScope],
) -> Result<(), DocumentationError> {
    let labels_are_exact = update["labels"].as_vec().is_some_and(
        |labels| matches!(labels.as_slice(), [label] if label.as_str() == Some("dependencies")),
    );
    let uniform = update["schedule"]["interval"].as_str() == Some("weekly")
        && update["open-pull-requests-limit"].as_i64() == Some(5)
        && labels_are_exact;
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
