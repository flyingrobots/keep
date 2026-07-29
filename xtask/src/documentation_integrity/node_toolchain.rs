//! This module owns the committed Node documentation-tool graph contract.

mod unique_json;

use serde_json::{Map, Value};

use crate::repository_file::RepositoryRoot;

use super::error::DocumentationError;
use super::repository_text::{self, RepositoryText};

const INSTALLER_PATH: &str = "scripts/install_documentation_tools.sh";
const INSTALLER_DIGEST: [u8; 32] = [
    0x12, 0xfb, 0x82, 0xcd, 0xdb, 0x65, 0x52, 0xe5, 0xae, 0xdb, 0x14, 0x54, 0x83, 0xf4, 0x8a, 0x8a,
    0x8b, 0x35, 0x54, 0xea, 0x2a, 0x90, 0xeb, 0xd4, 0x82, 0xd7, 0x4a, 0x61, 0x1f, 0xf6, 0xd3, 0xfd,
];
const LOCK_DIGEST: [u8; 32] = [
    0x74, 0x21, 0xce, 0x90, 0xdd, 0x52, 0x33, 0xfe, 0x99, 0x1a, 0x0b, 0x7e, 0xdd, 0xaa, 0xb7, 0x53,
    0x63, 0xf3, 0xad, 0x3b, 0x0f, 0x9e, 0x7d, 0xa2, 0xa4, 0x77, 0x65, 0xf0, 0x9c, 0x1d, 0xcb, 0x3b,
];
const LOCK_PATH: &str = "scripts/documentation-tools/package-lock.json";
const MANIFEST_PATH: &str = "scripts/documentation-tools/package.json";

pub(super) fn check(
    repository_root: &RepositoryRoot,
) -> Result<[RepositoryText; 3], DocumentationError> {
    let manifest = repository_text::read(repository_root, MANIFEST_PATH)?;
    let lock = repository_text::read(repository_root, LOCK_PATH)?;
    let installer = repository_text::read(repository_root, INSTALLER_PATH)?;
    admit(manifest.as_str(), lock.as_str(), installer.as_str())?;
    admit_lock_bytes(lock.as_str())?;
    manifest.verify(repository_root)?;
    lock.verify(repository_root)?;
    installer.verify(repository_root)?;
    Ok([manifest, lock, installer])
}

fn admit(manifest: &str, lock: &str, installer: &str) -> Result<(), DocumentationError> {
    let manifest = parse(MANIFEST_PATH, manifest)?;
    admit_manifest(&manifest)?;
    let lock = parse(LOCK_PATH, lock)?;
    admit_lock(&lock)?;
    admit_installer(installer)
}

fn admit_manifest(manifest: &Value) -> Result<(), DocumentationError> {
    if manifest.get("overrides").is_some() {
        return Err(contract(MANIFEST_PATH, "dependency overrides are absent"));
    }
    let observed = manifest
        .get("dependencies")
        .and_then(|dependencies| dependencies.get("markdownlint-cli2"))
        .and_then(Value::as_str);
    if observed == Some("0.23.2") {
        Ok(())
    } else {
        Err(DocumentationError::RepositoryValue {
            path: MANIFEST_PATH,
            field: "dependencies.markdownlint-cli2",
            expected: "0.23.2",
            observed: observed.map(str::to_owned),
        })
    }
}

fn admit_lock(lock: &Value) -> Result<(), DocumentationError> {
    if lock.get("lockfileVersion").and_then(Value::as_u64) != Some(3) {
        return Err(contract(LOCK_PATH, "lockfileVersion is exactly 3"));
    }
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
    if blake3::hash(installer.as_bytes()).as_bytes() == &INSTALLER_DIGEST {
        Ok(())
    } else {
        Err(contract(
            INSTALLER_PATH,
            "installer bytes match the reviewed digest",
        ))
    }
}

fn admit_lock_bytes(lock: &str) -> Result<(), DocumentationError> {
    if blake3::hash(lock.as_bytes()).as_bytes() == &LOCK_DIGEST {
        Ok(())
    } else {
        Err(contract(LOCK_PATH, "lock bytes match the reviewed digest"))
    }
}

fn parse(path: &'static str, raw: &str) -> Result<Value, DocumentationError> {
    unique_json::parse(raw).map_err(|source| DocumentationError::RepositoryJson { path, source })
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

const fn contract(path: &'static str, requirement: &'static str) -> DocumentationError {
    DocumentationError::RepositoryContract { path, requirement }
}

#[cfg(test)]
#[path = "node_toolchain/tests.rs"]
mod tests;
