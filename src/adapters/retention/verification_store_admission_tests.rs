//! Typed store contradictions stay distinct from inconclusive admission failures.

use super::filesystem_retention_test_fixture::{migrated_store, retention_witness};
use crate::{
    CatalogRestartByteLimit, CatalogRestartPolicy, FilesystemPlatformAdmissionError,
    FilesystemRetentionSnapshot, FilesystemRetentionSnapshotError, ReaderAttemptLimit,
    SegmentReadPolicy, StoreFormatMarkerDecodeError, StoreMigrationIntentDecodeError,
    StoreMigrationReceiptDecodeError, StoreRootIdentityCoordinate, VerificationError,
    VerificationObservation, VerificationRefusal, VerificationSource, VerificationSubject,
    VersionTwoRecordRefusal,
};
use std::os::unix::fs::MetadataExt;
use std::{error::Error, fs, io};

type ResultOf<T> = Result<T, Box<dyn Error>>;

// Size: medium. Oracle: bad literal record magic proves a content contradiction.
// Delete if a stronger public loader matrix preserves these typed causes.
#[test]
fn malformed_store_records_are_corrupt_with_the_original_magic() -> ResultOf<()> {
    for name in ["FORMAT", "migration.intent", "migration.receipt"] {
        let store = migrated_store("verification-malformed-store-record")?;
        let path = store.path().join(name);
        let mut bytes = fs::read(&path)?;
        *bytes.first_mut().ok_or("empty migration record")? ^= 1;
        let expected: [u8; 16] = bytes.get(..16).ok_or("missing magic")?.try_into()?;
        fs::write(&path, &bytes)?;
        let source = corrupt_cause(
            FilesystemRetentionSnapshot::load_for_verification(
                store.path(),
                policy()?,
                ReaderAttemptLimit::DEFAULT,
            )
            .err()
            .ok_or("bad magic admitted")?,
        )?;
        let typed = source
            .get_ref()
            .and_then(|source| source.downcast_ref::<VersionTwoRecordRefusal>());
        let observed = match typed {
            Some(VersionTwoRecordRefusal::Marker {
                source: StoreFormatMarkerDecodeError::InvalidMagic { observed },
            }) if name == "FORMAT" => observed,
            Some(VersionTwoRecordRefusal::Intent {
                source: StoreMigrationIntentDecodeError::InvalidMagic { observed },
            }) if name == "migration.intent" => observed,
            Some(VersionTwoRecordRefusal::Receipt {
                source: StoreMigrationReceiptDecodeError::InvalidMagic { observed },
            }) if name == "migration.receipt" => observed,
            _ => return Err(format!("{name} lost its exact decode refusal: {source:?}").into()),
        };
        assert_eq!(*observed, expected, "original magic for {name}");
        assert_eq!(fs::read(path)?, bytes, "refusal must preserve {name}");
    }
    Ok(())
}

// Size: medium. Oracle: copied migration records still name the donor's inode.
// Delete if migration identity binding is removed or a stronger loader law subsumes it.
#[test]
fn a_foreign_migration_binding_is_corruption_with_exact_root_coordinates() -> ResultOf<()> {
    let donor = migrated_store("verification-identity-donor")?;
    let recipient = migrated_store("verification-identity-recipient")?;
    for name in ["FORMAT", "migration.intent", "migration.receipt"] {
        fs::copy(donor.path().join(name), recipient.path().join(name))?;
    }
    let expected = fs::metadata(donor.path())?.ino();
    let observed = fs::metadata(recipient.path())?.ino();
    let before = retention_witness(recipient.path())?;
    let source = corrupt_cause(
        FilesystemRetentionSnapshot::load_for_verification(
            recipient.path(),
            policy()?,
            ReaderAttemptLimit::DEFAULT,
        )
        .err()
        .ok_or("foreign binding admitted")?,
    )?;
    let typed = source
        .get_ref()
        .and_then(|source| source.downcast_ref::<FilesystemPlatformAdmissionError>());
    assert!(
        matches!(typed, Some(FilesystemPlatformAdmissionError::RootIdentityChanged {
        coordinate: StoreRootIdentityCoordinate::File, expected: actual_expected, observed: actual_observed
    }) if *actual_expected == expected && *actual_observed == observed),
        "original binding coordinates required: {source:?}"
    );
    assert_eq!(
        retention_witness(recipient.path())?,
        before,
        "refusal must preserve the store"
    );
    Ok(())
}

fn corrupt_cause(error: VerificationError) -> ResultOf<io::Error> {
    let VerificationError::Refused {
        refusal,
        source: Some(source),
    } = error
    else {
        return Err(
            format!("typed store contradiction was not classified corrupt: {error:?}").into(),
        );
    };
    assert_eq!(
        refusal,
        VerificationRefusal::Corrupt {
            subject: VerificationSubject::PublishedView,
            expected: VerificationObservation::Canonical,
            observed: VerificationObservation::Refused,
        }
    );
    let VerificationSource::Retention(FilesystemRetentionSnapshotError::Admission { source }) =
        *source
    else {
        return Err("original admission boundary was lost".into());
    };
    Ok(source)
}

fn policy() -> ResultOf<CatalogRestartPolicy> {
    Ok(CatalogRestartPolicy::new(
        SegmentReadPolicy::MAXIMUM,
        CatalogRestartByteLimit::new(1_048_576)?,
    ))
}

// Size: small. Oracle: host-width exhaustion and unclassified I/O establish no
// contradiction. This is a public error-conversion law, not injected syscall evidence.
// Delete if admission errors gain a stronger typed operational boundary.
#[test]
fn inconclusive_admission_causes_remain_operational_without_losing_the_source() -> ResultOf<()> {
    let source = io::Error::new(
        io::ErrorKind::InvalidData,
        VersionTwoRecordRefusal::LengthOverflow { name: "FORMAT" },
    );
    let error = VerificationError::from(FilesystemRetentionSnapshotError::Admission { source });
    let source = operational_cause(error)?;
    assert!(
        matches!(
            source
                .get_ref()
                .and_then(|source| source.downcast_ref::<VersionTwoRecordRefusal>()),
            Some(VersionTwoRecordRefusal::LengthOverflow { name: "FORMAT" })
        ),
        "host-width cause must survive: {source:?}"
    );
    for kind in [io::ErrorKind::PermissionDenied, io::ErrorKind::InvalidData] {
        let source = io::Error::new(kind, "unclassified admission failure");
        let error = VerificationError::from(FilesystemRetentionSnapshotError::Admission { source });
        assert_eq!(
            operational_cause(error)?.kind(),
            kind,
            "I/O kind alone must not establish corruption"
        );
    }
    Ok(())
}

fn operational_cause(error: VerificationError) -> ResultOf<io::Error> {
    let VerificationError::Operational { source } = error else {
        return Err(
            format!("inconclusive admission was labeled content evidence: {error:?}").into(),
        );
    };
    let VerificationSource::Retention(FilesystemRetentionSnapshotError::Admission { source }) =
        *source
    else {
        return Err("original operational admission cause was lost".into());
    };
    Ok(source)
}
