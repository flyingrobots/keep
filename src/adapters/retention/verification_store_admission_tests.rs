//! Typed store contradictions stay distinct from inconclusive admission failures.

use super::filesystem_retention_test_fixture::initialized_store;
use crate::{
    CatalogRestartByteLimit, CatalogRestartPolicy, FilesystemRetentionSnapshot,
    FilesystemRetentionSnapshotError, ReaderAttemptLimit, SegmentReadPolicy,
    StoreFormatMarkerDecodeError, VerificationError, VerificationObservation, VerificationRefusal,
    VerificationSource, VerificationSubject, VersionTwoRecordRefusal,
};
use std::{error::Error, fs, io};

type ResultOf<T> = Result<T, Box<dyn Error>>;

// Size: medium. Oracle: bad literal record magic proves a content contradiction.
// Delete if a stronger public loader matrix preserves these typed causes.
#[test]
fn a_malformed_format_marker_is_corrupt_with_the_original_magic() -> ResultOf<()> {
    let store = initialized_store("verification-malformed-store-record")?;
    let path = store.path().join("FORMAT");
    let mut bytes = fs::read(&path)?;
    *bytes.first_mut().ok_or("empty format marker")? ^= 1;
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
    let Some(VersionTwoRecordRefusal::Marker {
        source: StoreFormatMarkerDecodeError::InvalidMagic { observed },
    }) = typed
    else {
        return Err(format!("FORMAT lost its exact decode refusal: {source:?}").into());
    };
    assert_eq!(*observed, expected, "original magic");
    assert_eq!(fs::read(path)?, bytes, "refusal must preserve FORMAT");
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
