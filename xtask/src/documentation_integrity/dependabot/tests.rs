use std::collections::BTreeSet;
use std::path::Path;

use crate::repository_file::RepositoryRoot;

use super::{DependencyScope, ScopeContract, workspace_members};

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

fn tracked() -> BTreeSet<DependencyScope> {
    [
        DependencyScope::new("cargo", "/"),
        DependencyScope::new("cargo", "/xtask"),
        DependencyScope::new("github-actions", "/"),
    ]
    .into_iter()
    .collect()
}

fn required() -> ScopeContract {
    ScopeContract::new(&tracked(), BTreeSet::new())
}

fn member_contract() -> ScopeContract {
    let members = [DependencyScope::new("cargo", "/xtask")].into();
    ScopeContract::new(&tracked(), members)
}

#[test]
fn complete_uniform_dependabot_policy_is_admitted() {
    assert!(super::admit(POLICY, &required()).is_ok());
}

#[test]
fn update_block_with_both_directory_forms_is_refused() {
    let policy = POLICY.replacen("      - /xtask\n", "", 1).replacen(
        "    schedule:\n",
        "    directory: /xtask\n    schedule:\n",
        1,
    );

    assert!(matches!(
        super::admit(&policy, &required()),
        Err(super::DocumentationError::RepositoryContract {
            path: super::DEPENDABOT_PATH,
            requirement: "update block chooses one directory form",
        })
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
fn duplicate_yaml_mapping_keys_are_refused_before_policy_admission() {
    let top_level = format!("{POLICY}updates: []\n");
    let nested = POLICY.replacen(
        "      interval: weekly\n",
        "      interval: weekly\n      interval: daily\n",
        1,
    );
    for policy in [top_level, nested] {
        assert!(matches!(
            super::admit(&policy, &required()),
            Err(super::DocumentationError::RepositoryYaml {
                path: super::DEPENDABOT_PATH,
                ..
            })
        ));
    }
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
fn prefix_confusable_maintenance_values_are_refused() {
    for policy in [
        POLICY.replacen("      interval: weekly", "      interval: weeklylies", 1),
        POLICY.replacen(
            "    open-pull-requests-limit: 5",
            "    open-pull-requests-limit: 50",
            1,
        ),
        POLICY.replacen("      - dependencies", "      - dependencies-extra", 1),
    ] {
        assert!(matches!(
            super::admit(&policy, &required()),
            Err(super::DocumentationError::RepositoryContractAt {
                path: super::DEPENDABOT_PATH,
                ref subject,
                requirement: "update block uses the maintenance policy",
            }) if subject == "cargo /"
        ));
    }
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

#[test]
fn workspace_members_need_no_update_scope_of_their_own() {
    let policy = POLICY.replace("      - /xtask\n", "");
    assert!(super::admit(&policy, &member_contract()).is_ok());
}

#[test]
fn a_workspace_member_listed_as_an_update_scope_is_refused() {
    assert!(matches!(
        super::admit(POLICY, &member_contract()),
        Err(super::DocumentationError::RepositoryContractAt {
            path: super::DEPENDABOT_PATH,
            ref subject,
            requirement: "workspace member is updated through its workspace root",
        }) if subject == "cargo /xtask"
    ));
}

#[test]
fn a_standalone_manifest_still_needs_its_own_update_scope() {
    let members = [DependencyScope::new("cargo", "/xtask")].into();
    let tracked = [
        DependencyScope::new("cargo", "/"),
        DependencyScope::new("cargo", "/fuzz"),
        DependencyScope::new("cargo", "/xtask"),
        DependencyScope::new("github-actions", "/"),
    ]
    .into();
    let policy = POLICY.replace("      - /xtask\n", "");
    assert!(matches!(
        super::admit(&policy, &ScopeContract::new(&tracked, members)),
        Err(super::DocumentationError::RepositoryContractAt {
            path: super::DEPENDABOT_PATH,
            ref subject,
            requirement: "tracked dependency scope has an update policy",
        }) if subject == "cargo /fuzz"
    ));
}

#[test]
fn workspace_members_exclude_the_root_and_name_exact_directories() {
    let manifest = "[package]\nname = \"keep\"\n\n[workspace]\nmembers = [\".\", \"xtask\", \"tools/a\"]\nresolver = \"3\"\n\n[workspace.lints.rust]\n";
    let expected: BTreeSet<_> = [
        DependencyScope::new("cargo", "/tools/a"),
        DependencyScope::new("cargo", "/xtask"),
    ]
    .into();
    assert_eq!(workspace_members::admit(manifest).ok(), Some(expected));
}

#[test]
fn unprovable_workspace_member_lists_are_refused() {
    for (manifest, expected) in [
        ("[package]\n", "workspace declares exactly one member list"),
        (
            "[workspace]\nresolver = \"3\"\n",
            "workspace declares exactly one member list",
        ),
        (
            "[workspace]\nmembers = [\"a\"]\nmembers = [\"b\"]\n",
            "workspace declares exactly one member list",
        ),
        (
            "[workspace]\nmembers = [\n  \"a\",\n]\n",
            "workspace members are one inline list",
        ),
        (
            "[workspace]\nmembers = [a]\n",
            "workspace member is one quoted path",
        ),
        (
            "[workspace]\nmembers = [\"crates/*\"]\n",
            "workspace member is one exact relative path",
        ),
        (
            "[workspace]\nmembers = [\"../a\"]\n",
            "workspace member is one exact relative path",
        ),
    ] {
        assert!(matches!(
            workspace_members::admit(manifest),
            Err(super::DocumentationError::RepositoryContract {
                path: workspace_members::ROOT_MANIFEST_PATH,
                requirement,
            }) if requirement == expected
        ));
    }
}
