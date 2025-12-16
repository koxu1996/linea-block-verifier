// Copyright (c) 2025 Merge Layers Inc.
//
// This source code is licensed under the Business Source License 1.1
// (the "License"); you may not use this file except in compliance with the
// License. You may obtain a copy of the License at
//
//     https://github.com/malda-protocol/linea-block-verifier/blob/master/LICENSE-BSL.md
//
// See the License for the specific language governing permissions and
// limitations under the License.
//

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
