//! Unsupported retention requests refuse independently of selected evidence.

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, initial_preparation, open_authority, root_pool_path,
};
use crate::{
    AdmittedRetentionRoot, CatalogRestartByteLimit, CatalogRestartPolicy,
    FilesystemRetentionSnapshot, ReaderAttemptLimit, RetentionNamespace, SegmentReadPolicy,
    VerificationDepth as Depth, VerificationError, VerificationRefusal, VerificationSubject,
    execute_retention_publication,
};
use std::{error::Error, fs};

// Size: medium. Oracle: the public supported-depth contract precedes evidence reads.
// Delete if a stronger public request-admission law subsumes both absent cases.
#[test]
fn unsupported_retention_requests_refuse_before_missing_evidence() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("verify-unsupported-depth")?;
    let bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&bytes)?;
    let _receipt = execute_retention_publication(&mut authority, &preparation)?;
    drop(authority);
    let root = AdmittedRetentionRoot::decode(&bytes)?;
    fs::remove_file(root_pool_path(sandbox.path(), &root))?;
    let policy = CatalogRestartPolicy::new(
        SegmentReadPolicy::MAXIMUM,
        CatalogRestartByteLimit::new(1_048_576)?,
    );
    let snapshot = FilesystemRetentionSnapshot::load_for_verification(
        sandbox.path(),
        policy,
        ReaderAttemptLimit::DEFAULT,
    )?;
    for namespace in [
        root.root().namespace().digest(),
        RetentionNamespace::try_from(b"absent".as_slice())?.digest(),
    ] {
        for requested in [
            Depth::ChunkIdentity,
            Depth::LayoutIdentity,
            Depth::CompleteBlobIdentity,
            Depth::CatalogReachability,
            Depth::SnapshotBinding,
        ] {
            let error = snapshot
                .verify_retention(namespace, requested)
                .err()
                .ok_or("unsupported request certified")?;
            assert!(
                matches!(&error, VerificationError::Refused { refusal: VerificationRefusal::Unsupported { subject: VerificationSubject::RetentionNamespace { namespace: actual }, requested: actual_requested, supported }, source: None } if *actual == namespace && *actual_requested == requested && *supported == [Depth::Framing, Depth::Checksum, Depth::RetentionClosure]),
                "unsupported request must precede evidence access: {error:?}"
            );
        }
    }
    Ok(())
}
