use std::error::Error;
use std::fs;
use std::path::Path;

use super::{CorpusError, CorpusStats, audit};
use crate::fuzz_campaign::policy::CampaignPolicy;
use crate::test_directory::TestDirectory;

#[test]
fn missing_corpus_is_normal_absence() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::create("missing-fuzz-corpus")?;
    let stats = audit(
        &directory.path().join("missing"),
        repository_root(),
        &policy()?,
    )?;
    assert_eq!(stats, CorpusStats { files: 0, bytes: 0 });
    directory.close()?;
    Ok(())
}

#[test]
fn unknown_target_directory_is_refused() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::create("unknown-fuzz-target")?;
    fs::create_dir(directory.path().join("unknown"))?;
    let result = audit(directory.path(), repository_root(), &policy()?);
    assert!(matches!(result, Err(CorpusError::UnexpectedTarget(_))));
    directory.close()?;
    Ok(())
}

#[test]
fn oversized_input_is_refused() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::create("oversized-fuzz-input")?;
    let target = directory.path().join("blob_hasher");
    fs::create_dir(&target)?;
    let oversized = target.join("oversized");
    fs::File::create(&oversized)?.set_len(policy()?.max_input_bytes() + 1)?;
    let result = audit(directory.path(), repository_root(), &policy()?);
    assert!(matches!(result, Err(CorpusError::InputBound { .. })));
    directory.close()?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn linked_input_is_refused() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::symlink;

    let directory = TestDirectory::create("linked-fuzz-input")?;
    let target = directory.path().join("blob_hasher");
    fs::create_dir(&target)?;
    let source = directory.path().join("source");
    fs::write(&source, b"seed")?;
    symlink(&source, target.join("linked"))?;
    let result = audit(directory.path(), repository_root(), &policy()?);
    assert!(matches!(result, Err(CorpusError::NonRegular(_))));
    directory.close()?;
    Ok(())
}

fn policy() -> Result<CampaignPolicy, Box<dyn Error>> {
    Ok(CampaignPolicy::load(repository_root())?)
}

fn repository_root() -> &'static Path {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().map_or(manifest, |parent| parent)
}
