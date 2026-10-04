//! This module owns the closed subject vocabulary registered by receipt v1.

use super::VerificationReceiptProjectionError as Error;
use crate::{BlobId, LayoutId, VerificationSubject};

/// A subject whose identity has a registered receipt-v1 slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptSubject {
    /// A complete logical content identity.
    Blob(BlobId),
    /// A canonical layout identity.
    Layout(LayoutId),
}

impl TryFrom<VerificationSubject> for ReceiptSubject {
    type Error = Error;
    fn try_from(subject: VerificationSubject) -> Result<Self, Error> {
        match subject {
            VerificationSubject::Blob { identity } => Ok(Self::Blob(identity)),
            VerificationSubject::Layout { identity } => Ok(Self::Layout(identity)),
            other => Err(Error::Subject(other)),
        }
    }
}

impl From<ReceiptSubject> for VerificationSubject {
    fn from(subject: ReceiptSubject) -> Self {
        match subject {
            ReceiptSubject::Blob(identity) => Self::Blob { identity },
            ReceiptSubject::Layout(identity) => Self::Layout { identity },
        }
    }
}
