//! Real root decoding followed by public verification classification.
#![allow(
    clippy::redundant_pub_crate,
    reason = "shared boundary assertions remain private to their integration-test binaries"
)]
use keep::{
    AdmittedRetentionRoot, RetentionRootDecodeError, VerificationError, VerificationRefusal,
    VerificationSource,
};
use std::error::Error;

type Outcome<'a> = Result<AdmittedRetentionRoot<'a>, RetentionRootDecodeError>;

// The wrapper adds a classification assertion, then returns the original typed
// cause to each unchanged precise corruption oracle. It never fabricates it.
pub(super) fn decode(bytes: &[u8]) -> Result<Outcome<'_>, Box<dyn Error>> {
    let error = match AdmittedRetentionRoot::decode(bytes) {
        Ok(root) => return Ok(Ok(root)),
        Err(error) => error,
    };
    let classified = VerificationError::from(error);
    let VerificationError::Refused {
        refusal: VerificationRefusal::Corrupt { .. },
        source: Some(source),
    } = classified
    else {
        return Err(format!("root contradiction misclassified: {classified:?}").into());
    };
    let VerificationSource::Root(error) = *source else {
        return Err("original root cause lost".into());
    };
    Ok(Err(error))
}
