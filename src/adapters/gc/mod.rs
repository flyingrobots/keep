//! Canonical garbage-collection record adapters for `keep.segment-store/v2`.
//!
//! This module owns the semantic GC retirement intent and receipt, their
//! canonical encoders, and their admitting decoders, and the deterministic
//! planner that classifies one physical inventory against one immutable
//! liveness snapshot, the fixed-phase execution protocol over a storage
//! port, and the recovery planner over the residue an interrupted execution
//! leaves. It does not own filesystem effects or reader fencing.

mod admitted_disposition;
mod admitted_intent;
mod admitted_receipt;
mod candidate;
mod canonical_disposition;
mod canonical_intent;
mod canonical_receipt;
mod disposition;
mod disposition_decode_error;
mod disposition_decoder;
mod disposition_encoder;
mod disposition_enums;
mod disposition_format;
mod evidence_digests;
mod execution;
mod execution_phase;
mod execution_storage;
mod filesystem_gc_authority;
mod filesystem_gc_error;
mod filesystem_gc_residue;
mod filesystem_gc_storage;
#[cfg(test)]
mod filesystem_gc_tests;
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
mod liveness_coordinates;
mod liveness_observation;
mod liveness_observation_error;
#[cfg(test)]
mod liveness_observation_tests;
mod liveness_snapshot;
mod plan;
mod plan_error;
mod plan_limits;
mod planner;
#[cfg(test)]
mod planner_tests;
mod reader_lock_device;
mod reader_lock_file;
mod reader_lock_identity;
mod reader_lock_mount;
mod receipt;
mod receipt_bytes;
mod receipt_decode_error;
mod receipt_decoder;
mod receipt_encoder;
mod receipt_format;
mod record_digests;
mod recovery_plan;
mod recovery_residue;
mod retained_closure;
mod retirement_intent;
mod segment_classification;
mod segment_pool_inventory;

pub use admitted_disposition::AdmittedRecoveryDispositionReceipt;
pub use admitted_intent::AdmittedGcRetirementIntent;
pub use admitted_receipt::AdmittedGcRetirementReceipt;
pub use candidate::GcCandidate;
pub use canonical_disposition::CanonicalRecoveryDispositionReceipt;
pub use canonical_intent::CanonicalGcRetirementIntent;
pub use canonical_receipt::CanonicalGcRetirementReceipt;
pub use disposition::{
    RecoveryDispositionArtifact, RecoveryDispositionCoordinates, RecoveryDispositionReceipt,
};
pub use disposition_decode_error::{RecoveryDispositionDecodeError, RecoveryDispositionField};
pub use disposition_enums::{
    RecoveryArtifactKind, RecoveryClassification, RecoveryDispositionDecision,
};
pub use evidence_digests::{
    ArtifactContentDigest, ArtifactIdentityDigest, CatalogSuccessorProofDigest,
    DecisionEvidenceDigest, DispositionSetDigest, ObservedHeadChecksum, PoolStateDigest,
    SegmentPoolIdentityDigest, VerificationEvidenceDigest,
};
pub use execution::{GcExecutionError, GcExecutionReceipt, execute_gc, resume_gc_execution};
pub use execution_phase::{GcExecutionPhase, GcExecutionPoint};
pub use execution_storage::GcExecutionStorage;
pub use filesystem_gc_authority::{FilesystemGcAuthority, GcRecoveryReport, PreparedGcExecution};
pub use filesystem_gc_error::FilesystemGcError;
pub(in crate::adapters) use filesystem_gc_residue::GC_ENTRY_NAMES;
pub use intent::GcRetirementIntent;
pub use intent_coordinates::GcRetirementIntentCoordinates;
pub use intent_decode_error::{GcRetirementIntentDecodeError, GcRetirementIntentEncodeError};
pub use intent_error::GcRetirementIntentError;
pub use liveness_coordinates::{GcLivenessCoordinates, GcRetentionState};
pub use liveness_observation::observe_gc_liveness;
pub(in crate::adapters) use liveness_observation::visit_retained_closures;
pub use liveness_observation_error::GcLivenessObservationError;
pub use liveness_snapshot::{GcLivenessSnapshot, GcLivenessSnapshotError};
pub use plan::{GcPlan, GcPlannedCandidate, GcPlannedSegment};
pub use plan_error::{GcPlanAmbiguity, GcPlanError};
pub use plan_limits::{GcLimits, GcLimitsError};
pub use planner::plan_gc;
pub use reader_lock_device::ReaderLockDevice;
pub use reader_lock_file::ReaderLockFile;
pub use reader_lock_identity::{ReaderLockCoordinate, ReaderLockIdentity};
pub use reader_lock_mount::ReaderLockMount;
pub use receipt::GcRetirementReceipt;
pub use receipt_decode_error::GcRetirementReceiptDecodeError;
pub use record_digests::{GcCandidateSetDigest, GcRetirementIntentDigest};
pub use recovery_plan::{
    GcFixedStage, GcRecoveryAmbiguity, GcRecoveryPlan, is_complete as is_gc_complete,
    plan_gc_recovery,
};
pub use recovery_residue::GcResidue;
pub use retained_closure::GcRetainedClosure;
pub use retirement_intent::{
    GcIntentDerivationError, GcIntentEvidence, catalog_successor_proof, derive_gc_intent,
    disposition_set_digest, post_retirement_pool_state, segment_pool_identity,
};
pub use segment_classification::{GcSegmentClassification, GcUnreachableEvidence};
pub(in crate::adapters) use segment_pool_inventory::read as read_segment_pool_inventory;
