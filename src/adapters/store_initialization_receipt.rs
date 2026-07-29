//! This module owns proof of complete store-root initialization.

/// Proof that ordered store initialization reached root synchronization.
///
/// Private fields prevent callers from manufacturing a successful receipt.
#[derive(Debug)]
#[must_use]
pub struct StoreInitializationReceipt {
    _private: (),
}

impl StoreInitializationReceipt {
    pub(super) const fn new() -> Self {
        Self { _private: () }
    }
}
