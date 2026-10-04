//! This module owns the independent reader oracle after retention crash recovery.

use std::io;
use std::path::Path;

use keep::{
    AdmittedRetentionRoot, CatalogRestartByteLimit, CatalogRestartPolicy,
    FilesystemRetentionSnapshot, ReaderAttemptLimit, SegmentReadPolicy,
};

use super::super::DurabilityCrashMatrixError;
use super::super::production_protocol::verification;

/// Reads product output independently of recovery receipts and forward retries.
/// The coordinate's publication prefix supplies the expected generation;
/// the frozen publication input supplies the namespace and exact root bytes.
pub(super) fn verify(
    store_root: &Path,
    expected_root: &[u8],
    expected_generation: Option<u64>,
) -> Result<(), DurabilityCrashMatrixError> {
    let limit = CatalogRestartByteLimit::new(1_048_576)
        .map_err(|source| verification("bound recovered catalog read", source))?;
    let policy = CatalogRestartPolicy::new(SegmentReadPolicy::MAXIMUM, limit);
    let view = FilesystemRetentionSnapshot::load(store_root, policy, ReaderAttemptLimit::DEFAULT)
        .map_err(|source| verification("load recovered retention snapshot", source))?;
    let observed_generation = view.retention_head().map(|head| head.generation().get());
    if observed_generation != expected_generation {
        return Err(verification(
            "verify recovered retention generation",
            io::Error::other(format!(
                "expected {expected_generation:?}, observed {observed_generation:?}"
            )),
        ));
    }
    let root = AdmittedRetentionRoot::decode(expected_root)
        .map_err(|source| verification("decode expected recovered root", source))?;
    let observed = view
        .retained_root(root.root().namespace().digest())
        .map_err(|source| verification("read recovered selected root", source))?;
    let expected = expected_generation.map(|_| expected_root);
    if observed.as_deref() != expected {
        return Err(verification(
            "verify recovered selected root bytes",
            io::Error::other(format!(
                "expected {expected:?}, observed {:?}",
                observed.as_deref()
            )),
        ));
    }
    Ok(())
}
