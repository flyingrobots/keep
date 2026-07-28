use std::fs;
use std::io::Write;

use crate::repository_file::RepositoryRoot;
use crate::test_directory::TestDirectory;

use super::{MAX_REPOSITORY_FILE_BYTES, read};

#[test]
fn repository_policy_reads_are_utf8_and_bounded() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::create("repository-text")?;
    fs::write(directory.path().join("policy.txt"), "policy\n")?;
    let root = RepositoryRoot::open(directory.path())?;

    assert_eq!(read(&root, "policy.txt")?, "policy\n");

    fs::write(directory.path().join("policy.txt"), [0xff])?;
    assert!(matches!(
        read(&root, "policy.txt"),
        Err(super::DocumentationError::RepositoryFileEncoding {
            path: "policy.txt",
            ..
        })
    ));
    drop(root);
    directory.close()?;
    Ok(())
}

#[test]
fn repository_policy_reads_refuse_bytes_beyond_the_bound() -> Result<(), Box<dyn std::error::Error>>
{
    let directory = TestDirectory::create("repository-text-bound")?;
    let path = directory.path().join("policy.txt");
    let mut file = fs::File::create(&path)?;
    let maximum = usize::try_from(MAX_REPOSITORY_FILE_BYTES)?;
    file.write_all(&vec![b'x'; maximum])?;
    file.write_all(b"x")?;
    drop(file);
    let root = RepositoryRoot::open(directory.path())?;

    assert!(matches!(
        read(&root, "policy.txt"),
        Err(super::DocumentationError::RepositoryFileTooLarge {
            path: "policy.txt",
            maximum: MAX_REPOSITORY_FILE_BYTES,
        })
    ));
    drop(root);
    directory.close()?;
    Ok(())
}
