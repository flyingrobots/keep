//! This module owns documentation-integrity orchestration regression evidence.

use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

use crate::repository_fixture::run_git;
use crate::test_directory::TestDirectory;

use super::{DocumentationError, check_with};

const LOCK_PATH: &str = "scripts/documentation-tools/package-lock.json";

#[test]
fn fixed_policy_replacement_during_tool_execution_is_refused() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::create("documentation-policy-replacement")?;
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| io::Error::other("workspace root is unavailable"))?;
    let source = source
        .to_str()
        .ok_or_else(|| io::Error::other("workspace root is not UTF-8"))?;
    run_git(
        directory.path(),
        &["clone", "--quiet", "--no-hardlinks", source, "repository"],
    )?;
    let repository = directory.path().join("repository");
    let result = check_with(&repository, |_, _, _, _| replace_lock(&repository));
    assert!(matches!(
        result,
        Err(DocumentationError::RepositoryFileChanged(LOCK_PATH))
    ));
    Ok(())
}

fn replace_lock(repository: &Path) -> Result<(), DocumentationError> {
    let lock = repository.join(LOCK_PATH);
    let replacement = repository.join("replacement-package-lock.json");
    fs::write(&replacement, "{}\n").map_err(inspect)?;
    fs::rename(replacement, lock).map_err(inspect)
}

fn inspect(source: io::Error) -> DocumentationError {
    DocumentationError::RepositoryFileInspect {
        path: LOCK_PATH,
        source,
    }
}
