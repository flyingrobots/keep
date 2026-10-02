//! These laws own namespace refusal before recovery changes retained evidence.

use std::error::Error;
use std::fs;
use std::io;

use super::filesystem_retention_test_fixture::{
    HEAD_HEX, MANIFEST_HEX, ROOT_HEX, fixture, initial_preparation, open_authority, refusal,
    retention_witness,
};
use super::{
    FilesystemRetentionRecoveryError, RetentionCurrentStateRefusal, RetentionPublicationStorage,
};

enum EntryPoint {
    Recovery,
    Publication,
}

// Size: medium. Oracle: unknown namespace state refuses before any mutation.
// Delete only when the protocol is removed or stronger filesystem evidence subsumes this law.
#[test]
fn direct_recovery_preserves_stages_when_namespace_admission_refuses() -> Result<(), Box<dyn Error>>
{
    require_namespace_refusal(EntryPoint::Recovery)
}

// Size: medium. Oracle: publication-triggered recovery has the same admission law.
// Delete only with its behavior or a stronger public-boundary replacement.
#[test]
fn publication_recovery_preserves_stages_when_namespace_admission_refuses()
-> Result<(), Box<dyn Error>> {
    require_namespace_refusal(EntryPoint::Publication)
}

fn require_namespace_refusal(entry_point: EntryPoint) -> Result<(), Box<dyn Error>> {
    let label = match entry_point {
        EntryPoint::Recovery => "direct",
        EntryPoint::Publication => "publication",
    };
    for (stage, hex) in [
        ("root.next", ROOT_HEX),
        ("manifest.next", MANIFEST_HEX),
        ("head.next", HEAD_HEX),
    ] {
        for intruder in [
            "foreign.dat",
            "roots/not-a-digest",
            "manifests/bogus.manifest",
        ] {
            let name = format!("namespace-{label}-{stage}-{}", intruder.replace('/', "-"));
            let (sandbox, mut authority) = open_authority(&name)?;
            let retention = sandbox.path().join("retention");
            let bytes = fixture(hex)?;
            fs::write(
                retention.join(stage),
                bytes.get(..8).ok_or("missing prefix")?,
            )?;
            fs::write(retention.join(intruder), b"unadmitted evidence")?;
            let before = retention_witness(sandbox.path())?;
            let root_bytes = fixture(ROOT_HEX)?;
            let preparation = initial_preparation(&root_bytes)?;

            let error = match entry_point {
                EntryPoint::Recovery => recovery_refusal(authority.recover())?,
                EntryPoint::Publication => {
                    RetentionPublicationStorage::verify_current(&mut authority, &preparation)
                        .err()
                        .ok_or("publication admitted an unknown namespace")?
                }
            };

            require_typed_refusal(&error, intruder)?;
            assert_eq!(
                retention_witness(sandbox.path())?,
                before,
                "namespace refusal must preserve all retained bytes for {stage} with {intruder}"
            );
        }
    }
    Ok(())
}

fn recovery_refusal(
    result: Result<super::RetentionRecoveryReceipt, FilesystemRetentionRecoveryError>,
) -> Result<io::Error, Box<dyn Error>> {
    match result {
        Err(FilesystemRetentionRecoveryError::Observe { source }) => Ok(source),
        observed => {
            Err(format!("namespace admission must refuse before recovery: {observed:?}").into())
        }
    }
}

fn require_typed_refusal(error: &io::Error, intruder: &str) -> Result<(), Box<dyn Error>> {
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    let matches = match (intruder, refusal(error)) {
        ("foreign.dat", Some(RetentionCurrentStateRefusal::UnknownRetentionEntry))
        | ("roots/not-a-digest", Some(RetentionCurrentStateRefusal::NonNamespaceEntry))
        | (
            "manifests/bogus.manifest",
            Some(RetentionCurrentStateRefusal::NoncanonicalPoolEntry {
                pool: "manifest pool",
            }),
        ) => true,
        _ => false,
    };
    assert!(
        matches,
        "exact namespace refusal missing for {intruder}: {error:?}"
    );
    Ok(())
}
