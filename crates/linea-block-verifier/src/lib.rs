//! Linea Block Verifier - Unified API
//!
//! This crate provides a single dependency for Linea block verification,
//! re-exporting both `linea-proof-core` and `linea-proof-host`.
//!
//! # Usage
//!
//! **zkVM guest:** `linea-block-verifier = { version = "0.1", features = ["std"] }`
//!
//! **Host application:** `linea-block-verifier = { version = "0.1", features = ["host"] }`

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use linea_proof_core as core;

#[cfg(feature = "host")]
pub use linea_proof_host as host;
