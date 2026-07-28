//! This module owns the CI documentation-job execution contract.

use yaml_rust2::YamlLoader;

use crate::repository_file::RepositoryRoot;

use super::error::DocumentationError;
use super::repository_text;

const CI_PATH: &str = ".github/workflows/ci.yml";
const MALFORMED_INPUT_COMMAND: &str = r"cargo test --locked --package xtask \
  documentation_integrity::execution::external_tests -- --ignored";
const XTASK_COMMAND: &str = "cargo xtask documentation-integrity-check";
const REVIEWED_RUNS: &[&str] = &[
    "rustup show",
    r#"documentation_tools="$RUNNER_TEMP/documentation-tools"
scripts/install_documentation_tools.sh "$documentation_tools"
printf '%s\n' \
  "$documentation_tools/bin" \
  "$documentation_tools/npm/node_modules/.bin" >> "$GITHUB_PATH""#,
    MALFORMED_INPUT_COMMAND,
    XTASK_COMMAND,
    r#"git diff --check "$(git hash-object -t tree /dev/null)" HEAD"#,
];

pub(super) fn check(repository_root: &RepositoryRoot) -> Result<(), DocumentationError> {
    let workflow = repository_text::read(repository_root, CI_PATH)?;
    admit(&workflow)
}

fn admit(workflow: &str) -> Result<(), DocumentationError> {
    let runs = documentation_runs(workflow)?;
    if !runs_are_reviewed(&runs) {
        return Err(contract(concat!(
            "documentation job run commands are reviewed and required ",
            "commands execute once"
        )));
    }
    Ok(())
}

fn documentation_runs(workflow: &str) -> Result<Vec<String>, DocumentationError> {
    let documents = YamlLoader::load_from_str(workflow).map_err(|source| {
        DocumentationError::RepositoryYaml {
            path: CI_PATH,
            source,
        }
    })?;
    let [document] = documents.as_slice() else {
        return Err(contract("workflow contains exactly one YAML document"));
    };
    let Some(steps) = document["jobs"]["documentation"]["steps"].as_vec() else {
        return Err(contract("workflow defines documentation job steps"));
    };
    let mut runs = Vec::new();
    for step in steps {
        let run = &step["run"];
        if run.is_badvalue() {
            continue;
        }
        if !step["if"].is_badvalue() {
            return Err(contract("documentation job run steps are unguarded"));
        }
        let Some(run) = run.as_str() else {
            return Err(contract("documentation job run values are strings"));
        };
        runs.push(run.trim_end_matches('\n').to_owned());
    }
    Ok(runs)
}

fn runs_are_reviewed(runs: &[String]) -> bool {
    runs.len() == REVIEWED_RUNS.len()
        && REVIEWED_RUNS
            .iter()
            .all(|required| runs.iter().filter(|run| run.as_str() == *required).count() == 1)
}

const fn contract(requirement: &'static str) -> DocumentationError {
    DocumentationError::RepositoryContract {
        path: CI_PATH,
        requirement,
    }
}

#[cfg(test)]
#[path = "workflow_contract/tests.rs"]
mod tests;
