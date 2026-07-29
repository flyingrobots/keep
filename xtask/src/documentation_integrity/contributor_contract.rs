//! This module owns contributor-facing documentation command contracts.

use crate::repository_file::RepositoryRoot;

use super::error::DocumentationError;
use super::repository_text::{self, RepositoryText};

const CONTRIBUTING_PATH: &str = "CONTRIBUTING.md";
const STANDARDS_PATH: &str = "docs/Documentation Standards.md";
const UNSTAGED_CHECK: &str = "git diff --check";
const STAGED_CHECK: &str = "git diff --cached --check";
const WHOLE_TREE_CHECK: &str = r#"git diff --check "$(git hash-object -t tree /dev/null)" HEAD"#;

pub(super) fn check(
    repository_root: &RepositoryRoot,
) -> Result<[RepositoryText; 2], DocumentationError> {
    Ok([
        checked_source(repository_root, CONTRIBUTING_PATH)?,
        checked_source(repository_root, STANDARDS_PATH)?,
    ])
}

fn checked_source(
    repository_root: &RepositoryRoot,
    path: &'static str,
) -> Result<RepositoryText, DocumentationError> {
    let raw = repository_text::read(repository_root, path)?;
    admit(path, raw.as_str())?;
    raw.verify(repository_root)?;
    Ok(raw)
}

fn admit(path: &'static str, raw: &str) -> Result<(), DocumentationError> {
    if raw.lines().any(|line| line == WHOLE_TREE_CHECK) {
        return Err(contract(
            path,
            "contributor command does not replace change checks with a whole-tree check",
        ));
    }
    require_line(
        path,
        raw,
        UNSTAGED_CHECK,
        "documents the unstaged whitespace check",
    )?;
    require_line(
        path,
        raw,
        STAGED_CHECK,
        "documents the staged whitespace check",
    )
}

fn require_line(
    path: &'static str,
    raw: &str,
    expected: &str,
    requirement: &'static str,
) -> Result<(), DocumentationError> {
    if raw.lines().any(|line| line == expected) {
        Ok(())
    } else {
        Err(contract(path, requirement))
    }
}

const fn contract(path: &'static str, requirement: &'static str) -> DocumentationError {
    DocumentationError::RepositoryContract { path, requirement }
}

#[cfg(test)]
mod tests {
    #[test]
    fn whole_tree_whitespace_replacement_is_refused() {
        let invalid = concat!(
            "git diff --check \"$(git hash-object -t tree /dev/null)\" HEAD\n",
            "git diff --check\n",
            "git diff --cached --check\n",
        );

        assert!(matches!(
            super::admit("guide.md", invalid),
            Err(super::DocumentationError::RepositoryContract {
                path: "guide.md",
                requirement: "contributor command does not replace change checks with a whole-tree check",
            })
        ));
    }

    #[test]
    fn contributor_commands_cover_staged_and_unstaged_whitespace() {
        for invalid in ["git diff --check\n", "git diff --cached --check\n"] {
            assert!(super::admit("guide.md", invalid).is_err());
        }
    }

    #[test]
    fn separate_change_checks_are_admitted() {
        let valid = "git diff --check\ngit diff --cached --check\n";

        assert!(super::admit("guide.md", valid).is_ok());
    }
}
