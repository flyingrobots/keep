//! Public verification preserves corruption classification for malformed retention records.

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, catalog_policy, fixture, initial_preparation, open_authority, retention_witness,
};
use crate::adapters::filesystem_exact_record::{ExactRecordError, ExactRecordRefusal};
use crate::{
    FilesystemRetentionSnapshot, ReaderAttemptLimit, RetentionViewError, VerificationError,
    VerificationObservation, VerificationRefusal, VerificationSource, VerificationSubject,
    execute_retention_publication,
};
use std::{error::Error, fs};

// Size: medium. Oracle: a truncated published head contradicts its fixed record length.
// Delete only when the format is retired or a stronger public verification law subsumes it.
#[test]
fn a_truncated_retention_head_remains_corruption_after_typed_error_conversion()
-> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("verification-truncated-retention-head")?;
    let bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&bytes)?;
    let _receipt = execute_retention_publication(&mut authority, &preparation)?;
    drop(authority);
    let path = sandbox.path().join("retention/HEAD");
    let mut head = fs::read(&path)?;
    head.pop().ok_or("head is empty")?;
    fs::write(path, head)?;
    let before = retention_witness(sandbox.path())?;

    let error = FilesystemRetentionSnapshot::load_for_verification(
        sandbox.path(),
        catalog_policy()?,
        ReaderAttemptLimit::DEFAULT,
    )
    .err()
    .ok_or("truncated published head was admitted")?;

    assert!(
        matches!(
            error,
            VerificationError::Refused {
                refusal: VerificationRefusal::Corrupt {
                    subject: VerificationSubject::PublishedView,
                    expected: VerificationObservation::Canonical,
                    observed: VerificationObservation::Refused,
                },
                source: Some(_),
            }
        ),
        "a demonstrated record contradiction must remain corruption: {error:?}"
    );
    let VerificationError::Refused {
        source: Some(source),
        ..
    } = &error
    else {
        return Err("missing refusal source".into());
    };
    let VerificationSource::View(RetentionViewError::Io { source }) = source.as_ref() else {
        return Err("missing observation boundary".into());
    };
    assert!(
        matches!(
            source
                .get_ref()
                .and_then(|source| source.downcast_ref::<ExactRecordError>()),
            Some(ExactRecordError::Refused(ExactRecordRefusal::KindOrLength))
        ),
        "the original typed record refusal must survive: {source:?}"
    );
    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "verification must preserve all retained evidence"
    );
    Ok(())
}
