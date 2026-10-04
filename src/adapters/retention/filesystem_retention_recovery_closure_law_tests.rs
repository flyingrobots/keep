//! These laws own semantic closure replay at recovered head publication.

use std::error::Error;

use super::filesystem_retention_recovery_history_tests::install_history;
use super::filesystem_retention_test_fixture::{
    ROOT_HEX, drive_publication, fixture, initial_preparation, open_authority, retention_witness,
};
use super::{AdmittedRetentionRoot, CanonicalRetentionRoot, FilesystemRetentionRecoveryError};
use crate::{
    LayoutId, RetentionAnchor, RetentionClosureCounter, RetentionClosureLimits,
    RetentionClosureVerificationError, RetentionPolicy, RetentionRoot, SegmentRecordIdentity,
};

// Size: medium. Oracle: every named closure member must exist in the current catalog.
// Delete only with this protocol or a stronger runtime-boundary replacement.
#[test]
fn recovered_head_refuses_an_absent_anchor_layout() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(ROOT_HEX)?;
    let template = AdmittedRetentionRoot::decode(&bytes)?;
    let anchor = template
        .root()
        .anchors()
        .first()
        .ok_or("missing fixture anchor")?;
    let mut text = anchor.layout_id().to_string();
    let last = text.pop().ok_or("empty layout identity")?;
    text.push(if last == '0' { '1' } else { '0' });
    let absent: LayoutId = text.parse()?;
    let root = RetentionRoot::new(
        template.root().namespace().clone(),
        template.root().generation(),
        RetentionPolicy::new(template.root().profile(), template.root().limits()),
        None,
        vec![RetentionAnchor::new(anchor.blob_id(), absent)],
    )?;
    require_closure_refusal(
        &CanonicalRetentionRoot::from_root(&root)?,
        |error| {
            matches!(error, RetentionClosureVerificationError::MissingMember { identity }
            if *identity == SegmentRecordIdentity::Layout(absent))
        },
        "absent-layout",
    )
}

// Size: medium. Oracle: stored physical closure limits apply during restart replay.
// Delete only with this protocol or a stronger runtime-boundary replacement.
#[test]
fn recovered_head_refuses_a_physical_closure_limit_violation() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(ROOT_HEX)?;
    let template = AdmittedRetentionRoot::decode(&bytes)?;
    let limits = template.root().limits();
    let smaller =
        RetentionClosureLimits::new(limits.nodes(), limits.depth(), limits.encoded_bytes(), 1)?;
    let root = RetentionRoot::new(
        template.root().namespace().clone(),
        template.root().generation(),
        RetentionPolicy::new(template.root().profile(), smaller),
        None,
        template.root().anchors().to_vec(),
    )?;
    require_closure_refusal(
        &CanonicalRetentionRoot::from_root(&root)?,
        |error| {
            matches!(error, RetentionClosureVerificationError::LimitExceeded {
            counter: RetentionClosureCounter::PhysicalBytes, maximum: 1, observed
        } if *observed > 1)
        },
        "physical-limit",
    )
}

fn require_closure_refusal(
    candidate: &CanonicalRetentionRoot,
    oracle: impl FnOnce(&RetentionClosureVerificationError) -> bool,
    label: &str,
) -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority(&format!("recovery-closure-law-{label}"))?;
    let bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&bytes)?;
    drive_publication(&mut authority, &preparation, 13)?;
    install_history(sandbox.path(), &preparation, candidate)?;
    let before = retention_witness(sandbox.path())?;
    let result = authority.recover();
    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "closure law {label} must preserve retained bytes: {result:?}"
    );
    let error = match &result {
        Err(FilesystemRetentionRecoveryError::Observe { source }) => source,
        other => {
            return Err(
                format!("closure law {label} must refuse during observation: {other:?}").into(),
            );
        }
    };
    let closure = error
        .get_ref()
        .and_then(|source| source.downcast_ref::<RetentionClosureVerificationError>())
        .ok_or("closure refusal lost its typed source")?;
    assert!(
        oracle(closure),
        "closure law {label} must preserve exact refusal: {result:?}"
    );
    Ok(())
}
