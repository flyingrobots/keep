//! This module owns the committed Node documentation-tool graph contract.

use serde_json::{Map, Value};

use crate::repository_file::RepositoryRoot;

use super::error::DocumentationError;
use super::repository_text;

const INSTALLER_PATH: &str = "scripts/install_documentation_tools.sh";
const LOCK_PATH: &str = "scripts/documentation-tools/package-lock.json";
const MANIFEST_PATH: &str = "scripts/documentation-tools/package.json";

pub(super) fn check(repository_root: &RepositoryRoot) -> Result<(), DocumentationError> {
    let manifest = repository_text::read(repository_root, MANIFEST_PATH)?;
    let lock = repository_text::read(repository_root, LOCK_PATH)?;
    let installer = repository_text::read(repository_root, INSTALLER_PATH)?;
    admit(&manifest, &lock, &installer)
}

fn admit(manifest: &str, lock: &str, installer: &str) -> Result<(), DocumentationError> {
    let manifest = parse(MANIFEST_PATH, manifest)?;
    admit_manifest(&manifest)?;
    let lock = parse(LOCK_PATH, lock)?;
    admit_lock(&lock)?;
    admit_installer(installer)
}

fn admit_manifest(manifest: &Value) -> Result<(), DocumentationError> {
    require(
        manifest.get("overrides").is_none(),
        MANIFEST_PATH,
        "dependency overrides are absent",
    )
}

fn admit_lock(lock: &Value) -> Result<(), DocumentationError> {
    require(
        lock.get("lockfileVersion").and_then(Value::as_u64) == Some(3),
        LOCK_PATH,
        "lockfileVersion is exactly 3",
    )?;
    let packages = lock.get("packages").and_then(Value::as_object).ok_or(
        DocumentationError::RepositoryContract {
            path: LOCK_PATH,
            requirement: "packages is an object",
        },
    )?;
    require_package_value(
        packages,
        "",
        &["dependencies", "markdownlint-cli2"],
        "packages[\"\"].dependencies.markdownlint-cli2",
        "0.23.2",
    )?;
    require_package_value(
        packages,
        "node_modules/markdownlint-cli2",
        &["dependencies", "js-yaml"],
        "packages[\"node_modules/markdownlint-cli2\"].dependencies.js-yaml",
        "5.2.2",
    )?;
    require_package_value(
        packages,
        "node_modules/js-yaml",
        &["version"],
        "packages[\"node_modules/js-yaml\"].version",
        "5.2.2",
    )?;
    require_package_value(
        packages,
        "node_modules/markdown-it",
        &["version"],
        "packages[\"node_modules/markdown-it\"].version",
        "14.3.0",
    )?;
    require_provenance(packages)?;
    Ok(())
}

fn admit_installer(installer: &str) -> Result<(), DocumentationError> {
    require(
        installer.contains("npm ci"),
        INSTALLER_PATH,
        "installation uses npm ci",
    )?;
    require(
        installer.contains("package-lock.json"),
        INSTALLER_PATH,
        "installation requires package-lock.json",
    )?;
    require(
        !installer.contains("npm install \\"),
        INSTALLER_PATH,
        "installation does not bypass the lock with npm install",
    )
}

fn parse(path: &'static str, raw: &str) -> Result<Value, DocumentationError> {
    serde_json::from_str(raw).map_err(|source| DocumentationError::RepositoryJson { path, source })
}

fn require_package_value(
    packages: &Map<String, Value>,
    package: &'static str,
    fields: &[&str],
    field: &'static str,
    expected: &'static str,
) -> Result<(), DocumentationError> {
    let mut value = packages.get(package);
    for field in fields {
        value = value.and_then(|current| current.get(field));
    }
    let observed = value.and_then(Value::as_str);
    if observed == Some(expected) {
        Ok(())
    } else {
        Err(DocumentationError::RepositoryValue {
            path: LOCK_PATH,
            field,
            expected,
            observed: observed.map(str::to_owned),
        })
    }
}

fn require_provenance(packages: &Map<String, Value>) -> Result<(), DocumentationError> {
    for (path, package) in packages {
        if !path.is_empty() {
            let object = package.as_object();
            if !object.is_some_and(|fields| {
                fields.contains_key("resolved") && fields.contains_key("integrity")
            }) {
                return Err(DocumentationError::RepositoryContractAt {
                    path: LOCK_PATH,
                    subject: path.clone(),
                    requirement: "package records resolved and integrity fields",
                });
            }
        }
    }
    Ok(())
}

fn require(
    condition: bool,
    path: &'static str,
    requirement: &'static str,
) -> Result<(), DocumentationError> {
    if condition {
        Ok(())
    } else {
        Err(DocumentationError::RepositoryContract { path, requirement })
    }
}

#[cfg(test)]
#[path = "node_toolchain/tests.rs"]
mod tests;
