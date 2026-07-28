use std::collections::BTreeSet;
use std::path::Path;

use crate::repository_file::RepositoryRoot;

use super::DependencyScope;

const POLICY: &str = r"version: 2
updates:
  - package-ecosystem: cargo
    directories:
      - /
      - /xtask
    schedule:
      interval: weekly
    open-pull-requests-limit: 5
    labels:
      - dependencies

  - package-ecosystem: github-actions
    directory: /
    schedule:
      interval: weekly
    open-pull-requests-limit: 5
    labels:
      - dependencies
";

fn required() -> BTreeSet<DependencyScope> {
    [
        DependencyScope::new("cargo", "/"),
        DependencyScope::new("cargo", "/xtask"),
        DependencyScope::new("github-actions", "/"),
    ]
    .into_iter()
    .collect()
}

#[test]
fn complete_uniform_dependabot_policy_is_admitted() {
    assert!(super::admit(POLICY, &required()).is_ok());
}

#[test]
fn list_termination_preserves_the_following_scope_declaration() {
    let block = [
        "  - package-ecosystem: cargo",
        "    directories:",
        "      - /",
        "    directory: /xtask",
    ];

    let scopes = super::block_scopes(&block);
    assert!(matches!(
        scopes,
        Ok(scopes) if scopes == vec![
            DependencyScope::new("cargo", "/"),
            DependencyScope::new("cargo", "/xtask"),
        ]
    ));
}

#[test]
fn missing_manifest_scope_is_refused() {
    let policy = POLICY.replace("      - /xtask\n", "");
    assert!(matches!(
        super::admit(&policy, &required()),
        Err(super::DocumentationError::RepositoryContractAt {
            path: super::DEPENDABOT_PATH,
            ref subject,
            requirement: "tracked dependency scope has an update policy",
        }) if subject == "cargo /xtask"
    ));
}

#[test]
fn duplicate_update_scope_is_refused() {
    let policy = POLICY.replace("      - /xtask\n", "      - /xtask\n      - /xtask\n");
    assert!(matches!(
        super::admit(&policy, &required()),
        Err(super::DocumentationError::RepositoryContractAt {
            path: super::DEPENDABOT_PATH,
            ref subject,
            requirement: "update scope appears exactly once",
        }) if subject == "cargo /xtask"
    ));
}

#[test]
fn nonuniform_maintenance_policy_is_refused() {
    let policy = POLICY.replacen("      interval: weekly", "      interval: daily", 1);
    assert!(matches!(
        super::admit(&policy, &required()),
        Err(super::DocumentationError::RepositoryContractAt {
            path: super::DEPENDABOT_PATH,
            ref subject,
            requirement: "update block uses the maintenance policy",
        }) if subject == "cargo /"
    ));
}

#[test]
fn committed_dependabot_policy_covers_every_tracked_manifest()
-> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("xtask manifest has no repository parent")?;
    let repository_root = RepositoryRoot::open(root)?;
    let process_directory = repository_root.process_directory()?;
    super::check(&repository_root, &process_directory)?;
    assert!(repository_root.is_current_path()?);
    Ok(())
}
