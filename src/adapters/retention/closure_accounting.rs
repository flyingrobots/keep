//! This module owns checked retention-closure resource accounting.

use crate::{
    RetentionClosureCounter, RetentionClosureLimits, RetentionClosureUsage,
    RetentionClosureVerificationError,
};

pub(super) struct ClosureAccounting {
    limits: RetentionClosureLimits,
    nodes: u64,
    maximum_depth: u16,
    encoded_bytes: u64,
    physical_bytes: u64,
}

impl ClosureAccounting {
    pub(super) const fn new(limits: RetentionClosureLimits) -> Self {
        Self {
            limits,
            nodes: 0,
            maximum_depth: 0,
            encoded_bytes: 0,
            physical_bytes: 0,
        }
    }

    pub(super) fn admit_depth(
        &mut self,
        observed: u16,
    ) -> Result<(), RetentionClosureVerificationError> {
        let maximum = self.limits.depth();
        if observed > maximum {
            return Err(RetentionClosureVerificationError::LimitExceeded {
                counter: RetentionClosureCounter::Depth,
                maximum: u64::from(maximum),
                observed: u64::from(observed),
            });
        }
        self.maximum_depth = self.maximum_depth.max(observed);
        Ok(())
    }

    pub(super) fn add_node(&mut self) -> Result<(), RetentionClosureVerificationError> {
        self.nodes = checked_add(
            RetentionClosureCounter::Nodes,
            self.nodes,
            1,
            self.limits.nodes(),
        )?;
        Ok(())
    }

    pub(super) fn add_encoded(
        &mut self,
        incoming: u64,
    ) -> Result<(), RetentionClosureVerificationError> {
        self.encoded_bytes = checked_add(
            RetentionClosureCounter::EncodedBytes,
            self.encoded_bytes,
            incoming,
            self.limits.encoded_bytes(),
        )?;
        Ok(())
    }

    pub(super) fn add_physical(
        &mut self,
        incoming: u64,
    ) -> Result<(), RetentionClosureVerificationError> {
        self.physical_bytes = checked_add(
            RetentionClosureCounter::PhysicalBytes,
            self.physical_bytes,
            incoming,
            self.limits.physical_bytes(),
        )?;
        Ok(())
    }

    pub(super) const fn usage(&self) -> RetentionClosureUsage {
        RetentionClosureUsage::from_verified(
            self.nodes,
            self.maximum_depth,
            self.encoded_bytes,
            self.physical_bytes,
        )
    }
}

fn checked_add(
    counter: RetentionClosureCounter,
    current: u64,
    incoming: u64,
    maximum: u64,
) -> Result<u64, RetentionClosureVerificationError> {
    let observed = current.checked_add(incoming).ok_or(
        RetentionClosureVerificationError::CounterOverflow {
            counter,
            current,
            incoming,
        },
    )?;
    if observed > maximum {
        return Err(RetentionClosureVerificationError::LimitExceeded {
            counter,
            maximum,
            observed,
        });
    }
    Ok(observed)
}
