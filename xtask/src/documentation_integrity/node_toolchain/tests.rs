use std::path::Path;

use crate::repository_file::RepositoryRoot;

const MANIFEST: &str = r#"{"dependencies":{"markdownlint-cli2":"0.23.2"}}"#;
const LOCK: &str = r#"{
  "lockfileVersion": 3,
  "packages": {
    "": {"dependencies": {"markdownlint-cli2": "0.23.2"}},
    "node_modules/markdownlint-cli2": {
      "dependencies": {"js-yaml": "5.2.2"},
      "resolved": "example",
      "integrity": "example"
    },
    "node_modules/js-yaml": {
      "version": "5.2.2",
      "resolved": "example",
      "integrity": "example"
    },
    "node_modules/markdown-it": {
      "version": "14.3.0",
      "resolved": "example",
      "integrity": "example"
    }
  }
}"#;
const INSTALLER: &str = "test -f package-lock.json\nnpm ci\n";

#[test]
fn admitted_node_toolchain_is_exact_and_lockfile_installed() {
    assert!(super::admit(MANIFEST, LOCK, INSTALLER).is_ok());
}

#[test]
fn dependency_overrides_are_refused() {
    let manifest = r#"{"overrides":{},"dependencies":{"markdownlint-cli2":"0.23.2"}}"#;
    assert!(matches!(
        super::admit(manifest, LOCK, INSTALLER),
        Err(super::DocumentationError::RepositoryContract {
            path: super::MANIFEST_PATH,
            requirement: "dependency overrides are absent",
        })
    ));
}

#[test]
fn manifest_dependency_version_drift_is_refused() {
    let manifest = r#"{"dependencies":{"markdownlint-cli2":"999.0.0"}}"#;
    assert!(matches!(
        super::admit(manifest, LOCK, INSTALLER),
        Err(super::DocumentationError::RepositoryValue {
            path: super::MANIFEST_PATH,
            field: "dependencies.markdownlint-cli2",
            expected: "0.23.2",
            observed: Some(ref observed),
        }) if observed == "999.0.0"
    ));
}

#[test]
fn duplicate_object_members_are_refused_at_every_depth() {
    let manifest = concat!(
        r#"{"dependencies":{"markdownlint-cli2":"999.0.0"},"#,
        r#""dependencies":{"markdownlint-cli2":"0.23.2"}}"#,
    );
    let lock = LOCK.replacen(
        r#""version": "14.3.0","#,
        r#""version": "999.0.0", "version": "14.3.0","#,
        1,
    );
    for result in [
        super::admit(manifest, LOCK, INSTALLER),
        super::admit(MANIFEST, &lock, INSTALLER),
    ] {
        assert!(matches!(
            result,
            Err(super::DocumentationError::RepositoryJson { .. })
        ));
    }
}

#[test]
fn dependency_version_drift_is_refused() {
    let lock = LOCK.replacen("\"5.2.2\"", "\"5.2.1\"", 1);
    let error = super::admit(MANIFEST, &lock, INSTALLER);
    assert!(matches!(
        &error,
        Err(super::DocumentationError::RepositoryValue {
            path: super::LOCK_PATH,
            field: "packages[\"node_modules/markdownlint-cli2\"].dependencies.js-yaml",
            expected: "5.2.2",
            observed: Some(observed),
        }) if observed == "5.2.1"
    ));
    assert_eq!(
        error.as_ref().map_err(ToString::to_string),
        Err(String::from(concat!(
            "repository file `scripts/documentation-tools/package-lock.json` requires ",
            "`packages[\"node_modules/markdownlint-cli2\"].dependencies.js-yaml` to be ",
            "\"5.2.2\"; observed \"5.2.1\""
        )))
    );
}

#[test]
fn missing_dependency_coordinate_is_refused_precisely() {
    let lock = LOCK.replacen("\"version\": \"14.3.0\"", "\"missing\": \"14.3.0\"", 1);
    assert!(matches!(
        super::admit(MANIFEST, &lock, INSTALLER),
        Err(super::DocumentationError::RepositoryValue {
            path: super::LOCK_PATH,
            field: "packages[\"node_modules/markdown-it\"].version",
            expected: "14.3.0",
            observed: None,
        })
    ));
}

#[test]
fn missing_package_provenance_is_refused() {
    let lock = LOCK.replacen("\"integrity\": \"example\"", "\"missing\": \"example\"", 1);
    let error = super::admit(MANIFEST, &lock, INSTALLER);
    assert!(matches!(
        &error,
        Err(super::DocumentationError::RepositoryContractAt {
            path: super::LOCK_PATH,
            subject,
            requirement: "package records resolved and integrity fields",
        }) if subject == "node_modules/markdownlint-cli2"
    ));
    let diagnostic = error.as_ref().map_err(ToString::to_string);
    assert!(diagnostic.is_err_and(|message| message.contains("node_modules/markdownlint-cli2")));
}

#[test]
fn unlocked_installer_is_refused() {
    assert!(matches!(
        super::admit(MANIFEST, LOCK, "npm install markdownlint-cli2\n"),
        Err(super::DocumentationError::RepositoryContract {
            path: super::INSTALLER_PATH,
            requirement: "installation uses npm ci",
        })
    ));
}

#[test]
fn committed_node_toolchain_satisfies_the_rust_law() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("xtask manifest has no repository parent")?;
    let repository_root = RepositoryRoot::open(root)?;
    super::check(&repository_root)?;
    assert!(repository_root.is_current_path()?);
    Ok(())
}
