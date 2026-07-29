use std::fs;

use crate::repository_file::RepositoryRoot;
use crate::test_directory::TestDirectory;

use super::super::{DocumentationError, INSTALLER_PATH, LOCK_PATH, MANIFEST_PATH};

const INSTALLER: &str = include_str!("../../../../../scripts/install_documentation_tools.sh");
const LOCK: &str = include_str!("../../../../../scripts/documentation-tools/package-lock.json");
const MANIFEST: &str = include_str!("../../../../../scripts/documentation-tools/package.json");

#[test]
fn unreviewed_lock_graph_changes_are_refused() -> Result<(), Box<dyn std::error::Error>> {
    let altered = LOCK.replacen(
        "\"resolved\": \"https://registry.npmjs.org/",
        "\"resolved\": \"https://example.com/",
        1,
    );
    let extra = LOCK.replacen(
        "\n  }\n}",
        concat!(
            ",\n",
            "    \"node_modules/unreviewed\": {\n",
            "      \"version\": \"1.0.0\",\n",
            "      \"resolved\": \"https://example.com/unreviewed.tgz\",\n",
            "      \"integrity\": \"sha512-example\"\n",
            "    }\n",
            "  }\n",
            "}"
        ),
        1,
    );

    for lock in [altered, extra] {
        assert!(matches!(
            check_with_lock(&lock)?,
            Err(DocumentationError::RepositoryContract {
                path: LOCK_PATH,
                requirement: "lock bytes match the reviewed digest",
            })
        ));
    }
    Ok(())
}

fn check_with_lock(
    lock: &str,
) -> Result<Result<(), DocumentationError>, Box<dyn std::error::Error>> {
    let repository = TestDirectory::create("node-lock-graph")?;
    write_repository(&repository, lock)?;
    let root = RepositoryRoot::open(repository.path())?;
    let result = super::super::check(&root).map(drop);
    repository.close()?;
    Ok(result)
}

fn write_repository(repository: &TestDirectory, lock: &str) -> Result<(), std::io::Error> {
    let tool_directory = repository.path().join("scripts/documentation-tools");
    fs::create_dir_all(&tool_directory)?;
    fs::write(repository.path().join(MANIFEST_PATH), MANIFEST)?;
    fs::write(repository.path().join(LOCK_PATH), lock)?;
    fs::write(repository.path().join(INSTALLER_PATH), INSTALLER)
}
