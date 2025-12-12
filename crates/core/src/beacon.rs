//! Beacon chain types for Linea.
//!
//! These types represent the beacon chain data structures.
//! When the `std` feature is enabled, serde serialization/deserialization is available.

use alloc::string::String;
use alloc::vec::Vec;
use alloy_primitives::{Address, B256, Bloom, Bytes};
use alloy_rlp::Encodable;

#[cfg(feature = "std")]
use crate::serde_utils::{
  deserialize_address_from_hex, deserialize_b256_from_hex, deserialize_bloom_from_hex,
  deserialize_bytes_from_hex, deserialize_u32_from_string, deserialize_u64_from_string,
  deserialize_vec_bytes_from_hex, serialize_address_as_hex, serialize_b256_as_hex,
  serialize_bloom_as_hex, serialize_bytes_as_hex, serialize_u32_as_string, serialize_u64_as_string,
  serialize_vec_bytes_as_hex,
};

/// Beacon block header message.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct BeaconHeader {
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_u64_as_string",
      deserialize_with = "deserialize_u64_from_string"
    )
  )]
  pub slot: u64,
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_u32_as_string",
      deserialize_with = "deserialize_u32_from_string"
    )
  )]
  pub round: u32,
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_u64_as_string",
      deserialize_with = "deserialize_u64_from_string"
    )
  )]
  pub timestamp: u64,
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_bytes_as_hex",
      deserialize_with = "deserialize_bytes_from_hex"
    )
  )]
  pub proposer_index: Bytes,
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_b256_as_hex",
      deserialize_with = "deserialize_b256_from_hex"
    )
  )]
  pub parent_root: B256,
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_b256_as_hex",
      deserialize_with = "deserialize_b256_from_hex"
    )
  )]
  pub state_root: B256,
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_b256_as_hex",
      deserialize_with = "deserialize_b256_from_hex"
    )
  )]
  pub body_root: B256,
}

impl Encodable for BeaconHeader {
  fn encode(&self, out: &mut dyn alloy_rlp::BufMut) {
    // Encode as: [slot, round, timestamp, [proposer], parent_root, state_root, body_root]
    // Note: Besu encodes slot/round/timestamp as fixed-size big-endian bytes, not RLP scalars
    alloy_rlp::Header {
      list: true,
      payload_length: self.payload_length(),
    }
    .encode(out);

    self.slot.to_be_bytes().as_slice().encode(out);
    self.round.to_be_bytes().as_slice().encode(out);
    self.timestamp.to_be_bytes().as_slice().encode(out);

    // Proposer is encoded as a list containing the address bytes
    alloy_rlp::Header {
      list: true,
      payload_length: self.proposer_index.length(),
    }
    .encode(out);
    self.proposer_index.as_ref().encode(out);

    self.parent_root.as_slice().encode(out);
    self.state_root.as_slice().encode(out);
    self.body_root.as_slice().encode(out);
  }

  fn length(&self) -> usize {
    alloy_rlp::Header {
      list: true,
      payload_length: self.payload_length(),
    }
    .length()
      + self.payload_length()
  }
}

impl BeaconHeader {
  fn payload_length(&self) -> usize {
    self.slot.to_be_bytes().as_slice().length()
      + self.round.to_be_bytes().as_slice().length()
      + self.timestamp.to_be_bytes().as_slice().length()
      + alloy_rlp::Header {
        list: true,
        payload_length: self.proposer_index.length(),
      }
      .length()
      + self.proposer_index.length()
      + self.parent_root.as_slice().length()
      + self.state_root.as_slice().length()
      + self.body_root.as_slice().length()
  }
}

/// Full beacon block (the "message" part of the API response).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct BeaconBlock {
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_u64_as_string",
      deserialize_with = "deserialize_u64_from_string"
    )
  )]
  pub slot: u64,
  pub proposer_index: String,
  pub body: BeaconBlockBody,
}

/// Beacon block body.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct BeaconBlockBody {
  pub execution_payload: ExecutionPayload,
  pub attestations: Vec<Attestation>,
}

/// Attestation in a beacon block.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct Attestation {
  pub data: AttestationData,
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_bytes_as_hex",
      deserialize_with = "deserialize_bytes_from_hex"
    )
  )]
  pub signature: Bytes,
}

/// Attestation data.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct AttestationData {
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_u64_as_string",
      deserialize_with = "deserialize_u64_from_string"
    )
  )]
  pub slot: u64,
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_b256_as_hex",
      deserialize_with = "deserialize_b256_from_hex"
    )
  )]
  pub beacon_block_root: B256,
}

/// Execution payload in a beacon block.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct ExecutionPayload {
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_b256_as_hex",
      deserialize_with = "deserialize_b256_from_hex"
    )
  )]
  pub parent_hash: B256,
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_address_as_hex",
      deserialize_with = "deserialize_address_from_hex"
    )
  )]
  pub fee_recipient: Address,
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_b256_as_hex",
      deserialize_with = "deserialize_b256_from_hex"
    )
  )]
  pub state_root: B256,
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_b256_as_hex",
      deserialize_with = "deserialize_b256_from_hex"
    )
  )]
  pub receipts_root: B256,
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_bloom_as_hex",
      deserialize_with = "deserialize_bloom_from_hex"
    )
  )]
  pub logs_bloom: Bloom,
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_b256_as_hex",
      deserialize_with = "deserialize_b256_from_hex"
    )
  )]
  pub prev_randao: B256,
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_u64_as_string",
      deserialize_with = "deserialize_u64_from_string"
    )
  )]
  pub block_number: u64,
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_u64_as_string",
      deserialize_with = "deserialize_u64_from_string"
    )
  )]
  pub gas_limit: u64,
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_u64_as_string",
      deserialize_with = "deserialize_u64_from_string"
    )
  )]
  pub gas_used: u64,
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_u64_as_string",
      deserialize_with = "deserialize_u64_from_string"
    )
  )]
  pub timestamp: u64,
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_bytes_as_hex",
      deserialize_with = "deserialize_bytes_from_hex"
    )
  )]
  pub extra_data: Bytes,
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_u64_as_string",
      deserialize_with = "deserialize_u64_from_string"
    )
  )]
  pub base_fee_per_gas: u64,
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_b256_as_hex",
      deserialize_with = "deserialize_b256_from_hex"
    )
  )]
  pub block_hash: B256,
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_vec_bytes_as_hex",
      deserialize_with = "deserialize_vec_bytes_from_hex"
    )
  )]
  pub transactions: Vec<Bytes>,
}

// =============================================================================
// API Response Wrappers
// =============================================================================
// These types match the JSON structure of Linea's beacon API responses.

/// Response wrapper for beacon header API.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct HeaderResponse {
  pub data: HeaderData,
}

/// Header data wrapper.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct HeaderData {
  pub message: BeaconHeader,
}

/// Response wrapper for beacon block API.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct BlockResponse {
  pub version: String,
  pub data: BlockData,
}

/// Block data wrapper (contains both message and signature).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct BlockData {
  pub message: BeaconBlock,
  #[cfg_attr(
    feature = "std",
    serde(
      serialize_with = "serialize_bytes_as_hex",
      deserialize_with = "deserialize_bytes_from_hex"
    )
  )]
  pub signature: Bytes,
}
