//! This boundary module owns the caller-supplied evidence digests a GC
//! record carries.
//!
//! Each value names one exact 32-byte BLAKE3-256 coordinate that a later GC
//! planner or executor derives. This slice admits and transports the bytes;
//! it does not recompute them, so every constructor is public and every type
//! is a distinct newtype so the coordinates cannot be swapped.

macro_rules! evidence_digest {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[must_use]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name([u8; 32]);

        impl $name {
            /// Wraps exact caller-supplied digest bytes.
            pub const fn new(bytes: [u8; 32]) -> Self {
                Self(bytes)
            }

            /// Returns the exact 32 digest bytes.
            #[must_use]
            pub const fn as_bytes(&self) -> &[u8; 32] {
                &self.0
            }
        }
    };
}

evidence_digest! {
    /// Digest of the complete verification evidence for one retirement
    /// candidate segment.
    VerificationEvidenceDigest
}

evidence_digest! {
    /// Digest of the complete verified proof that the catalog successor names
    /// no retirement candidate.
    CatalogSuccessorProofDigest
}

evidence_digest! {
    /// Identity digest of the exact admitted immutable segment pool.
    SegmentPoolIdentityDigest
}

evidence_digest! {
    /// Digest of the exact admitted recovery-disposition receipt set.
    DispositionSetDigest
}

evidence_digest! {
    /// Digest of the verified, synchronized segment pool after retirement.
    PoolStateDigest
}
