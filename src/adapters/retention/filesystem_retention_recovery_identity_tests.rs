//! These laws own pool inode admission before retention recovery publication.

use std::error::Error;
use std::fs;
use std::os::unix::fs::MetadataExt;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, drive_publication, fixture, head_path, initial_preparation, manifest_pool_path,
    open_authority, retention_witness, root_pool_path,
};
use super::{FilesystemRetentionRecoveryError, RetentionPool, RetentionRecoveryRefusal};

// Size: medium. Oracle: a retained stage's pool link must name the same inode.
// Delete only with the protocol or a stronger public-boundary replacement.
#[test]
fn substituted_root_pool_refuses_before_recovery_commits_head() -> Result<(), Box<dyn Error>> {
    require_substitution_refusal(RetentionPool::Roots)
}

// Size: medium. Oracle: identical bytes do not admit a substituted pool inode.
// Delete only with the protocol or a stronger public-boundary replacement.
#[test]
fn substituted_manifest_pool_refuses_before_recovery_commits_head() -> Result<(), Box<dyn Error>> {
    require_substitution_refusal(RetentionPool::Manifests)
}

fn require_substitution_refusal(pool: RetentionPool) -> Result<(), Box<dyn Error>> {
    let label = match pool {
        RetentionPool::Roots => "root",
        RetentionPool::Manifests => "manifest",
    };
    let (sandbox, mut authority) = open_authority(&format!("recovery-substituted-{label}"))?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    drive_publication(&mut authority, &preparation, 13)?;
    let target = match pool {
        RetentionPool::Roots => root_pool_path(sandbox.path(), preparation.candidate()),
        RetentionPool::Manifests => manifest_pool_path(sandbox.path(), &preparation),
    };
    let stage = sandbox
        .path()
        .join("retention")
        .join(format!("{label}.next"));
    let replacement = target.with_extension("replacement");
    fs::copy(&target, &replacement)?;
    fs::rename(replacement, &target)?;
    let source = fs::metadata(&stage)?;
    let replaced = fs::metadata(&target)?;
    if (source.dev(), source.ino()) == (replaced.dev(), replaced.ino())
        || fs::read(&stage)? != fs::read(&target)?
    {
        return Err("fixture must contain equal bytes on distinct inodes".into());
    }
    let before = retention_witness(sandbox.path())?;

    let result = authority.recover();

    assert!(
        !head_path(sandbox.path()).exists(),
        "recovery must not commit HEAD over a substituted {label} pool inode: {result:?}"
    );
    assert!(
        matches!(result, Err(FilesystemRetentionRecoveryError::Plan {
            source: RetentionRecoveryRefusal::PoolEntryDiffers { pool: observed }
        }) if observed == pool),
        "substituted {label} must refuse at pool admission: {result:?}"
    );
    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "substituted {label} refusal must preserve every retained byte"
    );
    Ok(())
}
