//! BoltFFI data bridge — single export location for Rust↔Kotlin bridge types.
//!
//! # Requirements
//! - [FR-039](crate) — MCP: server lifecycle
//! - [FR-049](crate) — Bridge: boltffi ↔ JNA wire format
//! - [FR-050](crate) — Bridge: rkyv serialization

pub(crate) mod ffi;
#[macro_use]
mod types;
pub(crate) mod core;
pub(crate) mod selection;
pub(crate) mod wire_format;

pub use core::TorvoxBridge;
pub use ffi::*;
pub use types::*;
pub use wire_format::*;

#[cfg(test)]
mod tests;
