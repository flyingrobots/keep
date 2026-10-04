//! Real decoder failures observed through the production verification collector.
#![allow(
    clippy::redundant_pub_crate,
    reason = "shared boundary assertions remain private to their integration-test binaries"
)]
use keep::{
    ReaderAttemptLimit, RetentionViewCoordinates, RetentionViewError, RetentionViewSource,
    VerificationError, VerificationRefusal, VerificationSource, VerificationSubject,
};
use std::{error::Error, io};

pub(super) fn classify<E: Error + Send + Sync + 'static>(
    cause: E,
    subject: VerificationSubject,
) -> Result<E, Box<dyn Error>> {
    let mut source = FailedObservation(Some(io::Error::new(io::ErrorKind::InvalidData, cause)));
    let error = keep::collect_verification_view(&mut source, ReaderAttemptLimit::DEFAULT)
        .err()
        .ok_or("malformed observed publication received a view")?;
    let VerificationError::Refused {
        refusal: VerificationRefusal::Corrupt {
            subject: actual, ..
        },
        source: Some(source),
    } = error
    else {
        return Err(format!("decoder contradiction misclassified: {error:?}").into());
    };
    assert_eq!(
        actual, subject,
        "classification must retain the observed subject"
    );
    let VerificationSource::View(RetentionViewError::Io { source }) = *source else {
        return Err("original observation cause lost".into());
    };
    Ok(*source
        .into_inner()
        .ok_or("typed decoder source absent")?
        .downcast::<E>()
        .map_err(|source| -> Box<dyn Error> { source })?)
}

struct FailedObservation(Option<io::Error>);
impl RetentionViewSource for FailedObservation {
    type View = ();
    fn coordinates(&mut self) -> io::Result<RetentionViewCoordinates> {
        Err(self
            .0
            .take()
            .ok_or_else(|| io::Error::other("unexpected repeat observation"))?)
    }
    fn load(&mut self) -> io::Result<()> {
        Err(io::Error::other(
            "malformed observation must prevent loading",
        ))
    }
}
