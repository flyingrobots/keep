//! Storage-independent retention namespace transition laws.

#[path = "retention_transition/refusal_laws.rs"]
mod refusal_laws;
mod support;

use std::io;

use keep::{
    AdmittedRetentionRoot, CanonicalRetentionRoot, RetentionGenerationExpectation,
    RetentionNamespace, RetentionRoot, RetentionRootDigest, RetentionTransitionReadiness,
    RootGeneration, plan_retention_transition,
};

const ONE_ANCHOR_ROOT: &str = include_str!("../conformance/segment-store/v2/one-anchor-root.hex");

#[test]
fn absent_namespace_admits_only_the_initial_candidate() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fixture_bytes()?;
    let candidate = AdmittedRetentionRoot::decode(&bytes)?;
    let readiness =
        plan_retention_transition(RetentionGenerationExpectation::Absent, None, candidate)?;
    assert!(matches!(
        readiness,
        RetentionTransitionReadiness::Publish { candidate }
            if candidate.root().generation().get() == 1
    ));
    Ok(())
}

#[test]
fn exact_successor_and_byte_identical_replay_have_distinct_readiness()
-> Result<(), Box<dyn std::error::Error>> {
    let initial_bytes = fixture_bytes()?;
    let current = AdmittedRetentionRoot::decode(&initial_bytes)?;
    let successor = successor(&current)?;
    let candidate = AdmittedRetentionRoot::decode(successor.encoded())?;
    let readiness = plan_retention_transition(
        RetentionGenerationExpectation::Current(current.root().generation()),
        Some(&current),
        candidate,
    )?;
    assert!(matches!(
        readiness,
        RetentionTransitionReadiness::Publish { candidate }
            if candidate.root().generation().get() == 2
    ));

    let published = AdmittedRetentionRoot::decode(successor.encoded())?;
    let replay = AdmittedRetentionRoot::decode(successor.encoded())?;
    let readiness = plan_retention_transition(
        RetentionGenerationExpectation::Current(current.root().generation()),
        Some(&published),
        replay,
    )?;
    assert!(matches!(
        readiness,
        RetentionTransitionReadiness::AlreadyCommitted { candidate }
            if candidate.encoded() == successor.encoded()
    ));
    Ok(())
}

fn successor(
    current: &AdmittedRetentionRoot<'_>,
) -> Result<CanonicalRetentionRoot, Box<dyn std::error::Error>> {
    candidate(
        current,
        current.root().namespace().as_bytes(),
        current.root().generation().successor()?,
        Some(current.digest()),
    )
}

fn candidate(
    current: &AdmittedRetentionRoot<'_>,
    namespace: &[u8],
    generation: RootGeneration,
    predecessor: Option<RetentionRootDigest>,
) -> Result<CanonicalRetentionRoot, Box<dyn std::error::Error>> {
    let root = RetentionRoot::new(
        RetentionNamespace::try_from(namespace)?,
        generation,
        keep::RetentionPolicy::new(current.root().profile(), current.root().limits()),
        predecessor,
        current.root().anchors().to_vec(),
    )?;
    CanonicalRetentionRoot::from_root(&root).map_err(Into::into)
}

fn fixture_bytes() -> Result<Vec<u8>, io::Error> {
    let encoded = ONE_ANCHOR_ROOT
        .strip_suffix('\n')
        .ok_or_else(|| io::Error::other("retention root fixture lacks final newline"))?;
    support::decode_hex(encoded)
}
