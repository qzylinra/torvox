//! BoltFFI data bridge — single export location for Rust↔Kotlin bridge types.
//!
//! # Requirements
//! - [FR-039](crate) — MCP: server lifecycle
//! - [FR-049](crate) — Bridge: boltffi ↔ JNA wire format
//! - [FR-050](crate) — Bridge: rkyv serialization

pub(crate) mod core;
pub(crate) mod ffi;
mod types;

pub(crate) use core::with_bridge;
pub use core::TorvoxBridge;
pub use ffi::*;
pub use types::*;

#[cfg(test)]
mod tests;
