//! This boundary module owns migration-recovery ambiguity formatting.

use std::error::Error;
use std::fmt;

use super::StoreMigrationRecoveryAmbiguity;

impl fmt::Display for StoreMigrationRecoveryAmbiguity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EffectBeforeIntent { effect } => write!(
                formatter,
                "migration {effect:?} effect exists without a durable intent"
            ),
            Self::StageUndecodable { stage } => {
                write!(
                    formatter,
                    "complete migration {stage:?} stage does not decode"
                )
            }
            Self::StageOverlong { stage, observed } => write!(
                formatter,
                "migration {stage:?} stage has {observed} bytes, more than its record"
            ),
            Self::StageDiffers { stage } => {
                write!(
                    formatter,
                    "migration {stage:?} stage bytes differ from their record"
                )
            }
            Self::IntentUndecodable { .. } => {
                formatter.write_str("migration intent does not decode")
            }
            Self::IntentDiffers => {
                formatter.write_str("migration intent binds a different store or root")
            }
            Self::NamespaceOutOfOrder { absent, present } => write!(
                formatter,
                "migration namespace slot {present} exists before slot {absent}"
            ),
            Self::MarkerBeforeNamespace => {
                formatter.write_str("format marker exists before the namespace prefix")
            }
            Self::MarkerUndecodable { .. } => formatter.write_str("format marker does not decode"),
            Self::ReceiptBeforeMarker => {
                formatter.write_str("migration receipt exists before the format marker")
            }
            Self::ReceiptUndecodable { .. } => {
                formatter.write_str("migration receipt does not bind the observed records")
            }
        }
    }
}

impl Error for StoreMigrationRecoveryAmbiguity {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IntentUndecodable { source } => Some(source),
            Self::MarkerUndecodable { source } => Some(source),
            Self::ReceiptUndecodable { source } => Some(source),
            _ => None,
        }
    }
}
