//! Filesystem retention capacity laws: orphan namespaces count against the ceiling.

use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::Path;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, initial_preparation, open_authority, retention_witness,
    successor_preparation, successor_root,
};
use super::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, RetentionPublicationStorage,
    RetentionTransitionDisposition,
};
use crate::{RetentionManifest, execute_retention_publication};

#[test]
fn a_full_namespace_pool_refuses_a_new_namespace_before_staging() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-capacity-full")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let candidate = AdmittedRetentionRoot::decode(&root_bytes)?;
    let own = namespace_hex(&candidate);
    create_orphans(sandbox.path(), RetentionManifest::MAXIMUM_ENTRY_COUNT, &own)?;
    let before = retention_witness(sandbox.path())?;
    let preparation = initial_preparation(&root_bytes)?;

    let error = RetentionPublicationStorage::verify_current(&mut authority, &preparation)
        .err()
        .ok_or("a 4,097th retention namespace was unexpectedly admitted")?;

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert_eq!(retention_witness(sandbox.path())?, before);
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_full_namespace_pool_admits_a_successor_in_an_existing_namespace() -> Result<(), Box<dyn Error>>
{
    let (sandbox, mut authority) = open_authority("filesystem-retention-capacity-existing")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let _published =
        execute_retention_publication(&mut authority, &initial_preparation(&root_bytes)?)?;
    let current = authority
        .observe_current()?
        .ok_or("published retention head was not observed")?;
    let current_root = AdmittedRetentionRoot::decode(&root_bytes)?;
    let current_manifest = AdmittedRetentionManifest::decode(current.manifest_bytes())?;
    create_orphans(
        sandbox.path(),
        RetentionManifest::MAXIMUM_ENTRY_COUNT - 1,
        &namespace_hex(&current_root),
    )?;
    let candidate = successor_root(&current_root)?;
    let preparation = successor_preparation(&current_root, &current_manifest, candidate.encoded())?;

    let disposition = RetentionPublicationStorage::verify_current(&mut authority, &preparation)?;

    assert_eq!(disposition, RetentionTransitionDisposition::Publish);
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

fn create_orphans(root: &Path, count: u32, exclude: &str) -> Result<(), Box<dyn Error>> {
    let roots = root.join("retention").join("roots");
    let wanted = usize::try_from(count)?;
    for name in (0_u64..)
        .map(synthetic_namespace)
        .filter(|name| name != exclude)
        .take(wanted)
    {
        fs::create_dir(roots.join(name))?;
    }
    Ok(())
}

fn synthetic_namespace(seed: u64) -> String {
    let mut name = String::with_capacity(64);
    let _ = write!(name, "{seed:016x}");
    name.push_str(&"0".repeat(48));
    name
}

fn namespace_hex(candidate: &AdmittedRetentionRoot<'_>) -> String {
    candidate
        .root()
        .namespace()
        .digest()
        .as_bytes()
        .iter()
        .fold(String::new(), |mut rendered, byte| {
            let _ = write!(rendered, "{byte:02x}");
            rendered
        })
}
