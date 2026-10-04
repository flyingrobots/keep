//! Interrupted retention stages cannot admit a complete zero generation.
//! Size: small; oracle: positive root/liveness generation domain contracts.
//! Delete only if a stronger public assessment test subsumes these refusals.

mod support;

use std::error::Error;

use keep::{
    LivenessGenerationError, RetentionHeadDecodeError, RetentionManifestDecodeError,
    RetentionRootDecodeError, RetentionStageAssessment, RootGenerationError, assess_head_stage,
    assess_manifest_stage, assess_root_stage,
};

const ROOT: &str = include_str!("../conformance/segment-store/v2/one-anchor-root.hex");
const MANIFEST: &str = include_str!("../conformance/segment-store/v2/one-root-manifest.hex");
const HEAD: &str = include_str!("../conformance/segment-store/v2/one-root-head.hex");

#[test]
fn a_complete_zero_root_generation_refuses_every_later_interrupted_prefix()
-> Result<(), Box<dyn Error>> {
    let mut bytes = support::decode_hex(ROOT.trim_end())?;
    bytes.get_mut(32..40).ok_or("missing generation")?.fill(0);
    for end in 40..bytes.len() {
        let prefix = bytes.get(..end).ok_or("missing root prefix")?;
        let assessment = assess_root_stage(Some(prefix));
        assert!(
            matches!(
                assessment,
                RetentionStageAssessment::Corrupt(RetentionRootDecodeError::Generation {
                    source: RootGenerationError::Zero
                })
            ),
            "zero root generation at prefix {end} must refuse, observed {assessment:?}"
        );
    }
    Ok(())
}

#[test]
fn a_complete_zero_manifest_generation_refuses_every_later_interrupted_prefix()
-> Result<(), Box<dyn Error>> {
    let mut bytes = support::decode_hex(MANIFEST.trim_end())?;
    bytes.get_mut(32..40).ok_or("missing generation")?.fill(0);
    for end in 40..bytes.len() {
        let prefix = bytes.get(..end).ok_or("missing manifest prefix")?;
        let assessment = assess_manifest_stage(Some(prefix));
        assert!(
            matches!(
                assessment,
                RetentionStageAssessment::Corrupt(
                    RetentionManifestDecodeError::LivenessGeneration {
                        source: LivenessGenerationError::Zero
                    }
                )
            ),
            "zero manifest generation at prefix {end} must refuse, observed {assessment:?}"
        );
    }
    Ok(())
}

#[test]
fn a_complete_zero_head_generation_refuses_every_later_interrupted_prefix()
-> Result<(), Box<dyn Error>> {
    let mut bytes = support::decode_hex(HEAD.trim_end())?;
    bytes.get_mut(24..32).ok_or("missing generation")?.fill(0);
    for end in 32..bytes.len() {
        let prefix = bytes.get(..end).ok_or("missing head prefix")?;
        let assessment = assess_head_stage(Some(prefix));
        assert!(
            matches!(
                assessment,
                RetentionStageAssessment::Corrupt(RetentionHeadDecodeError::LivenessGeneration {
                    source: LivenessGenerationError::Zero
                })
            ),
            "zero head generation at prefix {end} must refuse, observed {assessment:?}"
        );
    }
    Ok(())
}
