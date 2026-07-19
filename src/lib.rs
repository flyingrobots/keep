#![deny(warnings)]
#![forbid(unsafe_code)]
#![warn(clippy::cargo)]

//! Correctness-first content-addressed storage.
//!
//! Keep is at the repository-foundation stage. Its public storage API will be
//! introduced only after the content-identity, durable-format, and recovery
//! invariants have executable specifications.
