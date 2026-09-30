//! Canonical durable verification receipts for `keep.verification-receipt/v1`.
//!
//! A receipt is the replayable projection of one `VerificationReport` or
//! `VerificationRefusal` onto a fixed 384-byte checksummed record: the
//! subject, the admitted view coordinates, the depth established or the
//! stage refused, the refusal classification, and the exact layout and
//! target where the outcome binds them. It carries no plaintext, key
//! material, or path. Keep does not persist receipts; the application does.

mod canonical;
mod decode_error;
mod decoder;
mod decoder_semantics;
mod encoder;
mod enums;
mod format;
mod receipt;

pub use canonical::CanonicalVerificationReceipt;
pub use decode_error::{VerificationReceiptDecodeError, VerificationReceiptField};
pub use enums::{
    ReceiptEvidenceKind, ReceiptOutcomeKind, ReceiptRefusalClass, ReceiptSubjectKind,
    ReceiptViewKind,
};
pub use receipt::{
    ReceiptCorruption, ReceiptMissing, ReceiptRefusal, VERIFICATION_CONTRACT_VERSION,
    VerificationOutcome, VerificationReceipt, VerificationView,
};
