//! This module owns layout coordinates for domain storage-profile replay failures.

use crate::profile::StorageProfileVerifier;
use crate::{AdmittedLayout, LayoutId};

use super::ReconstructionFailure;

pub(super) struct ProfileVerifier<'a> {
    layout: LayoutId,
    verifier: StorageProfileVerifier<'a>,
}

impl<'a> ProfileVerifier<'a> {
    pub(super) fn new(
        layout: LayoutId,
        admitted: &'a AdmittedLayout,
    ) -> Result<Self, ReconstructionFailure> {
        let verifier = StorageProfileVerifier::new(admitted).map_err(|error| {
            ReconstructionFailure::Profile {
                layout,
                source: error,
            }
        })?;
        Ok(Self { layout, verifier })
    }

    pub(super) fn feed(&mut self, bytes: &[u8]) -> Result<(), ReconstructionFailure> {
        self.verifier
            .feed(bytes)
            .map_err(|error| ReconstructionFailure::Profile {
                layout: self.layout,
                source: error,
            })
    }

    pub(super) fn finish(self) -> Result<(), ReconstructionFailure> {
        self.verifier
            .finish()
            .map_err(|error| ReconstructionFailure::Profile {
                layout: self.layout,
                source: error,
            })
    }
}
