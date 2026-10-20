//! This executable owns the process-isolated durable locator laws.
//!
//! Child creation must not share a process with golden-store writer handoffs:
//! a child can inherit a live flock description until exec. Separate test
//! executables have separate descriptor tables even when Cargo runs in parallel.
//! The locator parent owns no store; each admitted child runs one law serially.

#![cfg(target_os = "linux")]

#[path = "durable_locator/suite.rs"]
mod suite;
