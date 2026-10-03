//! Existing layout-law inputs through public typed verification classification.
#![allow(
    clippy::redundant_pub_crate,
    reason = "shared boundary assertions remain private to their integration-test binaries"
)]
use keep::{LayoutDecodeError, VerificationError, VerificationRefusal, VerificationSource};
use std::error::Error;

pub(super) fn classified(error: LayoutDecodeError) -> Result<LayoutDecodeError, Box<dyn Error>> {
    let source = match VerificationError::from(error) {
        VerificationError::Refused {
            refusal: VerificationRefusal::Corrupt { .. },
            source: Some(source),
        } => source,
        VerificationError::Operational { source } => {
            assert!(
                matches!(
                    *source,
                    VerificationSource::Layout(LayoutDecodeError::ConfiguredEntryLimitExceeded {
                        maximum: 1,
                        observed: 2
                    })
                ),
                "only the specified one-entry resource cap is operational in this corpus"
            );
            source
        }
        other => {
            return Err(format!("layout contradiction lost its classification: {other:?}").into());
        }
    };
    let VerificationSource::Layout(source) = *source else {
        return Err("layout decoder cause lost".into());
    };
    Ok(source)
}
