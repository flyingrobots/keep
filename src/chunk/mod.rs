//! Deterministic physical chunk identity and boundary detection.
//!
//! This module owns `ChunkId` calculation and the fixed
//! `fastcdc-64k-v1` streaming detector. It does not own layouts, storage,
//! retention, profile selection, or codecs.

mod detector;
mod error;
mod gear_table;
mod hasher;
mod id;
mod length;
mod offset;
mod span;

pub use detector::FastCdc;
pub use error::{ChunkHashError, ChunkingError};
pub use id::ChunkId;
pub use length::ChunkLength;
pub use offset::ChunkOffset;
pub use span::ChunkSpan;
