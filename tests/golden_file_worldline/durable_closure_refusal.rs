//! Selected retained closure admission precedes every usable durable view.
//!
//! Size: medium. Oracle: every retained anchor requires its complete named closure.
//! The adversarial fixture is checksummed but semantically incomplete, installed
//! outside publication deliberately; production preflight must never publish it.
//! Delete only if durable retained closure admission is removed or subsumed.

use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use super::durable_fixture::{build_missing_chunk, identify, policy};
use keep::{
    CanonicalRetentionHead, CanonicalRetentionManifest, CanonicalRetentionRoot, DurableOutcome,
    DurableStore, DurableStoreError, LivenessGeneration, ReaderAttemptLimit,
    RegisteredRetentionProfile, RetentionAnchor, RetentionClosureLimits,
    RetentionClosureVerificationError, RetentionHead, RetentionManifest, RetentionManifestEntry,
    RetentionManifestLength, RetentionNamespace, RetentionNamespaceDigest, RetentionPolicy,
    RetentionRoot, RootGeneration, SegmentRecordIdentity,
};

struct InstalledClaim {
    namespace: RetentionNamespaceDigest,
    evidence: Vec<PathBuf>,
}

#[test]
fn an_unsatisfied_retained_closure_refuses_snapshot_admission_before_output()
-> Result<(), Box<dyn Error>> {
    let bytes = b"retained root claims a missing chunk";
    let sandbox = build_missing_chunk("durable-incomplete-retained-closure", bytes)?;
    let named = identify(bytes)?;
    let missing = named.spans.first().ok_or("chunk absent")?.id();
    let InstalledClaim {
        namespace,
        evidence,
    } = install_claim(
        sandbox.path(),
        RetentionAnchor::new(named.target, named.record.id()),
    )?;
    let before = evidence
        .iter()
        .map(fs::read)
        .collect::<Result<Vec<_>, _>>()?;
    let store = DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT)?;
    let failure = store
        .snapshot()
        .err()
        .ok_or("incomplete retained closure admitted a snapshot")?;
    assert!(
        matches!(&failure, DurableStoreError::Closure { namespace: actual, source }
        if *actual == namespace && matches!(source.as_ref(),
            RetentionClosureVerificationError::MissingMember { identity }
            if *identity == SegmentRecordIdentity::Chunk(missing))),
        "snapshot must refuse exact retained closure member: {failure:?}"
    );
    let mut output = vec![0xAB];
    let failure = store
        .reconstruct(named.target, &mut output)
        .err()
        .ok_or("incomplete retained closure reconstructed")?;
    assert!(
        matches!(&failure, DurableOutcome::Store(DurableStoreError::Closure { namespace: actual, source })
        if *actual == namespace && matches!(source.as_ref(),
            RetentionClosureVerificationError::MissingMember { identity }
            if *identity == SegmentRecordIdentity::Chunk(missing))),
        "convenience read must preserve snapshot refusal: {failure:?}"
    );
    assert_eq!(
        output,
        [0xAB],
        "admission refusal must not touch caller output"
    );
    assert_eq!(
        evidence
            .iter()
            .map(fs::read)
            .collect::<Result<Vec<_>, _>>()?,
        before,
        "admission refusal must preserve selected root, manifest and head bytes"
    );
    Ok(())
}

fn install_claim(path: &Path, anchor: RetentionAnchor) -> Result<InstalledClaim, Box<dyn Error>> {
    let namespace = RetentionNamespace::try_from(&b"incomplete"[..])?;
    let digest = namespace.digest();
    let root = RetentionRoot::new(
        namespace,
        RootGeneration::INITIAL,
        RetentionPolicy::new(
            RegisteredRetentionProfile::SINGLE_CANONICAL_WITNESS_V1,
            RetentionClosureLimits::new(4096, 8, 16_777_216, 67_108_864)?,
        ),
        None,
        vec![anchor],
    )?;
    let root_bytes = CanonicalRetentionRoot::from_root(&root)?;
    let manifest = RetentionManifest::new(
        LivenessGeneration::INITIAL,
        None,
        vec![RetentionManifestEntry::new(
            digest,
            RootGeneration::INITIAL,
            root_bytes.digest(),
        )],
    )?;
    let manifest_bytes = CanonicalRetentionManifest::from_manifest(&manifest)?;
    let head = RetentionHead::new(
        LivenessGeneration::INITIAL,
        RetentionManifestLength::new(u64::try_from(manifest_bytes.encoded().len())?)?,
        manifest_bytes.digest(),
        None,
    )?;
    let directory = path.join("retention/roots").join(hex(digest.as_bytes())?);
    fs::create_dir(&directory)?;
    let root_path = directory.join(format!(
        "0000000000000001-{}.root",
        hex(root_bytes.digest().as_bytes())?
    ));
    let manifest_path = path.join("retention/manifests").join(format!(
        "0000000000000001-{}.manifest",
        hex(manifest_bytes.digest().as_bytes())?
    ));
    let head_path = path.join("retention/HEAD");
    fs::write(&root_path, root_bytes.encoded())?;
    fs::write(&manifest_path, manifest_bytes.encoded())?;
    fs::write(
        &head_path,
        CanonicalRetentionHead::from_head(&head).encoded(),
    )?;
    Ok(InstalledClaim {
        namespace: digest,
        evidence: vec![root_path, manifest_path, head_path],
    })
}

fn hex(bytes: &[u8]) -> Result<String, std::fmt::Error> {
    let mut result = String::new();
    for byte in bytes {
        write!(result, "{byte:02x}")?;
    }
    Ok(result)
}
