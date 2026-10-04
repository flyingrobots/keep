//! Closure resource-limit precedence laws.

use std::error::Error;

use keep::{
    RetentionClosureCounter, RetentionClosureLimits, RetentionClosureVerificationError,
    RetentionRoot,
};

use super::{
    one_zero_bundle::{root_with_limits, verify_bundle},
    support::require_error,
};

#[test]
fn depth_refusal_precedes_second_node_admission() -> Result<(), Box<dyn Error>> {
    let root = root_with_limits(RetentionClosureLimits::new(2, 1, 220, 509)?, None)?;

    assert_limit(&root, RetentionClosureCounter::Depth, 1, 2)
}

#[test]
fn node_refusal_follows_depth_admission_and_precedes_chunk_lookup() -> Result<(), Box<dyn Error>> {
    let root = root_with_limits(RetentionClosureLimits::new(1, 2, 220, 509)?, None)?;

    assert_limit(&root, RetentionClosureCounter::Nodes, 1, 2)
}

#[test]
fn physical_byte_refusal_precedes_layout_decoding() -> Result<(), Box<dyn Error>> {
    let root = root_with_limits(RetentionClosureLimits::new(2, 2, 220, 363)?, None)?;

    assert_limit(&root, RetentionClosureCounter::PhysicalBytes, 363, 364)
}

#[test]
fn encoded_byte_refusal_follows_layout_record_charge() -> Result<(), Box<dyn Error>> {
    let root = root_with_limits(RetentionClosureLimits::new(2, 2, 219, 509)?, None)?;

    assert_limit(&root, RetentionClosureCounter::EncodedBytes, 219, 220)
}

fn assert_limit(
    root: &RetentionRoot,
    counter: RetentionClosureCounter,
    maximum: u64,
    observed: u64,
) -> Result<(), Box<dyn Error>> {
    let error = require_error(
        verify_bundle(root)?,
        "resource-constrained closure unexpectedly verified",
    )?;
    assert!(matches!(
        error,
        RetentionClosureVerificationError::LimitExceeded {
            counter: actual_counter,
            maximum: actual_maximum,
            observed: actual_observed,
        } if actual_counter == counter
            && actual_maximum == maximum
            && actual_observed == observed
    ));
    Ok(())
}
