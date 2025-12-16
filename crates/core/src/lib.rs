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

//! Core verification logic for Linea V4 proofs.
//!
//! This crate contains no-std compatible code that can run inside a zkVM.
//! It has no networking or I/O dependencies.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod constants;
pub mod error;
#[cfg(feature = "std")]
pub mod serde_utils;
pub mod types;
mod verify;
#[cfg(feature = "std")]
pub mod zkvm;

// Only expose the core verification functions at the top level
pub use verify::{verify_block_hash, verify_header};

use alloc::vec::Vec;
use alloy_primitives::{B256, keccak256};
use alloy_rlp::Encodable;

/// Compute beacon slot from block number based on Linea's activation block.
///
/// In Linea, there is no parent_beacon_block_root (EIP-4788) - it is set to zeroes.
/// Each slot corresponds to one block.
///
/// - Mainnet: activation_block = 24787631, so slot 1 = block 24787632
/// - Sepolia: activation_block = 18665903, so slot 1 = block 18665904
///
/// Returns an error if the chain ID is unsupported or the block number is at or before
/// the activation block.
pub fn beacon_slot_from_block_number(
  block_number: u64,
  chain_id: u64,
) -> Result<u64, error::VerifyError> {
  let activation_block = match chain_id {
    constants::LINEA_MAINNET_CHAIN_ID => constants::LINEA_MAINNET_ACTIVATION_BLOCK,
    constants::LINEA_SEPOLIA_CHAIN_ID => constants::LINEA_SEPOLIA_ACTIVATION_BLOCK,
    _ => return Err(error::VerifyError::UnsupportedChainId(chain_id)),
  };

  if block_number <= activation_block {
    return Err(error::VerifyError::BlockAtOrBeforeActivation {
      block_number,
      activation_block,
    });
  }

  Ok(block_number - activation_block)
}

/// Compute beacon_block_root using the same method as Maru:
/// beacon_block_root = keccak256(rlp_encode([slot, round, timestamp, [proposer], parent_root, state_root, body_root]))
pub fn compute_beacon_block_root(header: &types::BeaconHeader) -> B256 {
  let mut rlp_buffer = Vec::new();
  header.encode(&mut rlp_buffer);
  keccak256(&rlp_buffer)
}

/// Compute body hash using the same method as Maru:
/// body = rlp([[attestation_sigs], execution_payload])
/// body_root = keccak256(body)
///
/// The `prev_block_attestation` is the attestation for the previous slot.
/// For slot 1 (the first slot), there is no previous attestation.
pub fn compute_body_hash(
  prev_block_attestation: Option<&types::Attestation>,
  execution_payload: &types::ExecutionPayload,
) -> B256 {
  use alloy_rlp::BufMut;

  let mut body_buffer = Vec::new();
  let mut temp_buffer = Vec::new();

  // First element: list of attestation signatures (prevCommitSeals)
  let mut seals_list_buffer = Vec::new();
  if let Some(attestation) = prev_block_attestation {
    // Each seal is a list containing just the signature bytes
    let mut seal_buffer = Vec::new();
    attestation.signature.as_ref().encode(&mut seal_buffer);

    // Wrap in list
    alloy_rlp::Header {
      list: true,
      payload_length: seal_buffer.len(),
    }
    .encode(&mut seals_list_buffer);
    seals_list_buffer.put_slice(&seal_buffer);
  }

  // Encode the seals list header
  alloy_rlp::Header {
    list: true,
    payload_length: seals_list_buffer.len(),
  }
  .encode(&mut temp_buffer);
  temp_buffer.put_slice(&seals_list_buffer);

  // Second element: execution payload
  let mut payload_buffer = Vec::new();

  // Encode execution payload fields
  execution_payload
    .parent_hash
    .as_slice()
    .encode(&mut payload_buffer);
  execution_payload
    .fee_recipient
    .as_slice()
    .encode(&mut payload_buffer);
  execution_payload
    .state_root
    .as_slice()
    .encode(&mut payload_buffer);
  execution_payload
    .receipts_root
    .as_slice()
    .encode(&mut payload_buffer);
  execution_payload
    .logs_bloom
    .as_slice()
    .encode(&mut payload_buffer);
  execution_payload
    .prev_randao
    .as_slice()
    .encode(&mut payload_buffer);

  // Numbers encoded as fixed-size 8-byte values (like in header)
  execution_payload
    .block_number
    .to_be_bytes()
    .as_slice()
    .encode(&mut payload_buffer);
  execution_payload
    .gas_limit
    .to_be_bytes()
    .as_slice()
    .encode(&mut payload_buffer);
  execution_payload
    .gas_used
    .to_be_bytes()
    .as_slice()
    .encode(&mut payload_buffer);
  execution_payload
    .timestamp
    .to_be_bytes()
    .as_slice()
    .encode(&mut payload_buffer);

  execution_payload
    .extra_data
    .as_ref()
    .encode(&mut payload_buffer);

  // base_fee_per_gas: convert to minimal big-endian representation (strip leading zeros)
  let base_fee_bytes = execution_payload.base_fee_per_gas.to_be_bytes();
  let first_nonzero = base_fee_bytes
    .iter()
    .position(|&b| b != 0)
    .unwrap_or(base_fee_bytes.len() - 1);
  base_fee_bytes[first_nonzero..].encode(&mut payload_buffer);

  execution_payload
    .block_hash
    .as_slice()
    .encode(&mut payload_buffer);

  // Transactions list
  let mut tx_list_buffer = Vec::new();
  for tx in &execution_payload.transactions {
    tx.as_ref().encode(&mut tx_list_buffer);
  }
  alloy_rlp::Header {
    list: true,
    payload_length: tx_list_buffer.len(),
  }
  .encode(&mut payload_buffer);
  payload_buffer.put_slice(&tx_list_buffer);

  // Wrap execution payload in list
  alloy_rlp::Header {
    list: true,
    payload_length: payload_buffer.len(),
  }
  .encode(&mut temp_buffer);
  temp_buffer.put_slice(&payload_buffer);

  // Wrap entire body in outer list
  alloy_rlp::Header {
    list: true,
    payload_length: temp_buffer.len(),
  }
  .encode(&mut body_buffer);
  body_buffer.put_slice(&temp_buffer);

  keccak256(&body_buffer)
}
