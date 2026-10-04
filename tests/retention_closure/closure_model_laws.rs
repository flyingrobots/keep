//! Exhaustive closure-accounting outcomes against a boring model.

use std::error::Error;

use keep::{RetentionClosureCounter, RetentionClosureLimits, RetentionClosureVerificationError};

use super::one_zero_bundle::{root_with_limits, verify_bundle};

const NODE_LIMITS: [u64; 3] = [1, 2, 3];
const DEPTH_LIMITS: [u16; 3] = [1, 2, 3];
const ENCODED_LIMITS: [u64; 3] = [219, 220, 221];
const PHYSICAL_LIMITS: [u64; 5] = [363, 364, 508, 509, 510];

#[test]
fn exhaustive_boundary_policies_agree_with_the_boring_model() -> Result<(), Box<dyn Error>> {
    for nodes in NODE_LIMITS {
        for depth in DEPTH_LIMITS {
            for encoded in ENCODED_LIMITS {
                for physical in PHYSICAL_LIMITS {
                    let limits = RetentionClosureLimits::new(nodes, depth, encoded, physical)?;
                    let root = root_with_limits(limits, None)?;
                    let observed = classify(verify_bundle(&root)?)?;
                    let expected = model(nodes, depth, encoded, physical);

                    assert_eq!(
                        observed, expected,
                        "nodes={nodes} depth={depth} encoded={encoded} physical={physical}"
                    );
                }
            }
        }
    }
    Ok(())
}

fn model(nodes: u64, depth: u16, encoded: u64, physical: u64) -> Outcome {
    if physical < 364 {
        return Outcome::limit(RetentionClosureCounter::PhysicalBytes, physical, 364);
    }
    if encoded < 220 {
        return Outcome::limit(RetentionClosureCounter::EncodedBytes, encoded, 220);
    }
    if depth < 2 {
        return Outcome::limit(RetentionClosureCounter::Depth, u64::from(depth), 2);
    }
    if nodes < 2 {
        return Outcome::limit(RetentionClosureCounter::Nodes, nodes, 2);
    }
    if physical < 509 {
        return Outcome::limit(RetentionClosureCounter::PhysicalBytes, physical, 509);
    }
    Outcome::Verified {
        nodes: 2,
        depth: 2,
        encoded: 220,
        physical: 509,
    }
}

fn classify(
    result: Result<keep::VerifiedRetentionClosure, RetentionClosureVerificationError>,
) -> Result<Outcome, Box<dyn Error>> {
    match result {
        Ok(evidence) => Ok(Outcome::Verified {
            nodes: evidence.usage().node_count(),
            depth: evidence.usage().maximum_depth(),
            encoded: evidence.usage().encoded_bytes(),
            physical: evidence.usage().physical_bytes(),
        }),
        Err(RetentionClosureVerificationError::LimitExceeded {
            counter,
            maximum,
            observed,
        }) => Ok(Outcome::limit(counter, maximum, observed)),
        Err(error) => Err(error.into()),
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Outcome {
    Limit {
        counter: RetentionClosureCounter,
        maximum: u64,
        observed: u64,
    },
    Verified {
        nodes: u64,
        depth: u16,
        encoded: u64,
        physical: u64,
    },
}

impl Outcome {
    const fn limit(counter: RetentionClosureCounter, maximum: u64, observed: u64) -> Self {
        Self::Limit {
            counter,
            maximum,
            observed,
        }
    }
}
