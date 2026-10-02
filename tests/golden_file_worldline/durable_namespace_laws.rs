//! Selected retention roots must belong to the namespace selecting them.
//!
//! Size: medium (owned production-profile filesystem). Oracle: a manifest
//! entry binds namespace, root generation, and root digest jointly. A canonical
//! foreign root is contradictory evidence, even when its closure is complete.
//! Delete only if namespace selection is removed or stronger public laws subsume it.

use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use keep::{
    CanonicalRetentionHead, CanonicalRetentionManifest, DurableOutcome, DurableStore,
    DurableStoreError, FilesystemRetentionSnapshot, FilesystemRetentionSnapshotError,
    ReaderAttemptLimit, RetentionHead, RetentionManifest, RetentionManifestEntry,
    RetentionManifestLength, RetentionNamespace, RetentionNamespaceDigest,
};

use super::durable_fixture::{build, identify, policy};

#[test]
fn a_selected_root_from_another_namespace_refuses_direct_read() -> Result<(), Box<dyn Error>> {
    let sandbox = build("durable-direct-foreign-root", &[b"namespace-bound"])?;
    let namespace = install_foreign_selection(sandbox.path())?;
    let snapshot =
        FilesystemRetentionSnapshot::load(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT)?;

    let refusal = snapshot
        .retained_root(namespace)
        .err()
        .ok_or("foreign namespace root was returned by the public reader")?;

    assert_root_refusal(&refusal, namespace)?;
    Ok(())
}

#[test]
fn a_foreign_retained_namespace_refuses_durable_output() -> Result<(), Box<dyn Error>> {
    let bytes = b"namespace-bound";
    let sandbox = build("durable-foreign-root-output", &[bytes])?;
    let namespace = install_foreign_selection(sandbox.path())?;
    let store = DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT);
    let refusal = store
        .snapshot()
        .err()
        .ok_or("foreign namespace root admitted a durable snapshot")?;
    let DurableStoreError::Snapshot(source) = refusal else {
        return Err(format!("unexpected snapshot refusal: {refusal:?}").into());
    };
    assert_root_refusal(&source, namespace)?;
    let mut output = vec![0xAB];
    let refusal = store
        .reconstruct(identify(bytes)?.target, &mut output)
        .err()
        .ok_or("foreign namespace root produced durable output")?;
    let DurableOutcome::Store(DurableStoreError::Snapshot(source)) = refusal else {
        return Err(format!("unexpected read refusal: {refusal:?}").into());
    };
    assert_root_refusal(&source, namespace)?;
    assert_eq!(
        output,
        [0xAB],
        "namespace refusal must precede caller output"
    );
    Ok(())
}

fn assert_root_refusal(
    refusal: &FilesystemRetentionSnapshotError,
    _namespace: RetentionNamespaceDigest,
) -> Result<(), Box<dyn Error>> {
    assert!(
        matches!(refusal, FilesystemRetentionSnapshotError::Root { source }
        if source.kind() == ErrorKind::InvalidData),
        "a namespace contradiction must report root admission refusal: {refusal:?}"
    );
    Ok(())
}

fn install_foreign_selection(path: &Path) -> Result<RetentionNamespaceDigest, Box<dyn Error>> {
    let view = FilesystemRetentionSnapshot::load(path, policy()?, ReaderAttemptLimit::DEFAULT)?;
    let selected = view
        .manifest()
        .ok_or("manifest absent")?
        .entries()
        .first()
        .copied()
        .ok_or("root entry absent")?;
    let original = view
        .retained_root(selected.namespace())?
        .ok_or("root absent")?;
    let head = *view.retention_head().ok_or("retention head absent")?;
    drop(view);
    let namespace = RetentionNamespace::try_from(&b"different-namespace"[..])?.digest();
    let manifest = RetentionManifest::new(
        head.generation(),
        None,
        vec![RetentionManifestEntry::new(
            namespace,
            selected.root_generation(),
            selected.root_digest(),
        )],
    )?;
    let manifest = CanonicalRetentionManifest::from_manifest(&manifest)?;
    let head = RetentionHead::new(
        head.generation(),
        RetentionManifestLength::new(u64::try_from(manifest.encoded().len())?)?,
        manifest.digest(),
        None,
    )?;
    let directory = path
        .join("retention/roots")
        .join(hex(namespace.as_bytes())?);
    fs::create_dir(&directory)?;
    fs::write(
        directory.join(format!(
            "{:016x}-{}.root",
            selected.root_generation().get(),
            hex(selected.root_digest().as_bytes())?
        )),
        original,
    )?;
    fs::write(
        path.join("retention/manifests").join(format!(
            "{:016x}-{}.manifest",
            head.generation().get(),
            hex(manifest.digest().as_bytes())?
        )),
        manifest.encoded(),
    )?;
    fs::write(
        path.join("retention/HEAD"),
        CanonicalRetentionHead::from_head(&head).encoded(),
    )?;
    Ok(namespace)
}

fn hex(bytes: &[u8]) -> Result<String, std::fmt::Error> {
    let mut result = String::new();
    for byte in bytes {
        write!(result, "{byte:02x}")?;
    }
    Ok(result)
}
