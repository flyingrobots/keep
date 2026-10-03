//! This boundary module owns durable read receipts: the reference receipt
//! plus the view it was established against.

use super::DurableView;
use crate::{RangeReadReceipt, ReconstructionReceipt};

/// One authenticated complete reconstruction against one durable view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use = "the receipt records the authenticated identity, length, and view"]
pub struct DurableReconstructionReceipt {
    receipt: ReconstructionReceipt,
    view: DurableView,
}

impl DurableReconstructionReceipt {
    pub(super) const fn new(receipt: ReconstructionReceipt, view: DurableView) -> Self {
        Self { receipt, view }
    }

    /// The target, exact layout, and emitted length authenticated.
    pub const fn receipt(self) -> ReconstructionReceipt {
        self.receipt
    }

    /// The view the reconstruction was established against.
    #[must_use]
    pub const fn view(self) -> DurableView {
        self.view
    }
}

/// One authenticated exact range read against one durable view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use = "the receipt records the authenticated range, layout, and view"]
pub struct DurableRangeReadReceipt {
    receipt: RangeReadReceipt,
    view: DurableView,
}

impl DurableRangeReadReceipt {
    pub(super) const fn new(receipt: RangeReadReceipt, view: DurableView) -> Self {
        Self { receipt, view }
    }

    /// The target, exact layout, requested range, and emitted length.
    pub const fn receipt(self) -> RangeReadReceipt {
        self.receipt
    }

    /// The view the range was established against.
    #[must_use]
    pub const fn view(self) -> DurableView {
        self.view
    }
}
