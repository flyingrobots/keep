//! Canonical garbage-collection record adapters for `keep.segment-store/v2`.
//!
//! This module owns the semantic GC retirement intent and receipt, their
//! canonical encoders, and their admitting decoders. It does not own GC
//! planning, execution, reader fencing, recovery, or the recovery-disposition
//! receipt, whose registered enumerations are not yet frozen in the format
//! definition.

mod admitted_intent;
mod admitted_receipt;
mod candidate;
mod canonical_intent;
mod canonical_receipt;
mod evidence_digests;
mod intent;
mod intent_candidate_decoder;
mod intent_coordinates;
mod intent_decode_error;
mod intent_decode_error_display;
mod intent_decoder;
mod intent_encoder;
mod intent_error;
mod intent_field_decoder;
mod intent_format;
mod intent_header_decoder;
mod intent_integrity;
mod intent_semantic_header;
mod reader_lock_identity;
mod receipt;
mod receipt_bytes;
mod receipt_decode_error;
mod receipt_decoder;
mod receipt_encoder;
mod receipt_format;
mod record_digests;

pub use admitted_intent::AdmittedGcRetirementIntent;
pub use admitted_receipt::AdmittedGcRetirementReceipt;
pub use candidate::GcCandidate;
pub use canonical_intent::CanonicalGcRetirementIntent;
pub use canonical_receipt::CanonicalGcRetirementReceipt;
pub use evidence_digests::{
    CatalogSuccessorProofDigest, DispositionSetDigest, PoolStateDigest, SegmentPoolIdentityDigest,
    VerificationEvidenceDigest,
};
pub use intent::GcRetirementIntent;
pub use intent_coordinates::GcRetirementIntentCoordinates;
pub use intent_decode_error::{GcRetirementIntentDecodeError, GcRetirementIntentEncodeError};
pub use intent_error::GcRetirementIntentError;
pub use reader_lock_identity::{ReaderLockCoordinate, ReaderLockIdentity};
pub use receipt::GcRetirementReceipt;
pub use receipt_decode_error::GcRetirementReceiptDecodeError;
pub use record_digests::{GcCandidateSetDigest, GcRetirementIntentDigest};
