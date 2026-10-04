//! This module owns admitted recovery-stage metadata.

use super::{RecoveryStage, RecoveryStageLength, RecoveryStageMetadataError};

/// Name-selected fixed stage with an admitted metadata byte length.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryStageMetadata {
    stage: RecoveryStage,
    length: RecoveryStageLength,
}

impl RecoveryStageMetadata {
    /// Admits one metadata byte length under its fixed-name protocol maximum.
    ///
    /// # Errors
    ///
    /// Returns [`RecoveryStageMetadataError`] when `observed` exceeds the
    /// maximum selected by `stage`.
    pub const fn new(
        stage: RecoveryStage,
        observed: u64,
    ) -> Result<Self, RecoveryStageMetadataError> {
        let maximum = stage.maximum_length();
        if observed > maximum {
            Err(RecoveryStageMetadataError::Oversized {
                stage,
                maximum,
                observed,
            })
        } else {
            Ok(Self {
                stage,
                length: RecoveryStageLength::from_validated(observed),
            })
        }
    }

    /// Returns the fixed stage selected by the canonical name.
    #[must_use]
    pub const fn stage(self) -> RecoveryStage {
        self.stage
    }

    /// Returns the admitted metadata byte length.
    pub const fn length(self) -> RecoveryStageLength {
        self.length
    }
}
