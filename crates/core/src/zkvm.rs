//! zkVM input/output serialization utilities.
//!
//! This module provides functionality to serialize verification inputs
//! for use in zkVM environments and deserialize them for verification.
//!
//! Only available when the `std` feature is enabled.

use alloy_consensus::Header;
use alloy_primitives::B256;
use serde::{Deserialize, Serialize};

use crate::beacon::{BlockResponse, HeaderResponse};
use crate::constants::LineaNetwork;
use crate::error::VerifyError;
use crate::verify::{verify_block_hash, verify_header};

/// Verification mode for zkVM.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum VerifyMode {
  /// Verify only the block hash (faster, smaller input).
  BlockHash,
  /// Verify the full header with Linea bug detection.
  FullHeader,
}

/// Input for zkVM block hash verification.
///
/// This is the minimal input for verifying a block hash.
/// Use this when you only need to prove a block hash is authentic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyBlockHashInput {
  /// The block hash to verify.
  pub block_hash: B256,
  /// The beacon block response.
  pub block_response: BlockResponse,
  /// The beacon header response.
  pub header_response: HeaderResponse,
  /// The network.
  pub network: LineaNetwork,
}

impl VerifyBlockHashInput {
  /// Create a new block hash verification input.
  pub fn new(
    block_hash: B256,
    block_response: BlockResponse,
    header_response: HeaderResponse,
    network: LineaNetwork,
  ) -> Self {
    Self {
      block_hash,
      block_response,
      header_response,
      network,
    }
  }

  /// Serialize this input to bytes for passing into zkVM.
  pub fn to_bytes(&self) -> Result<Vec<u8>, VerifyError> {
    bincode::serialize(self).map_err(|_| VerifyError::SerializationError)
  }

  /// Deserialize input from bytes inside zkVM.
  pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
    bincode::deserialize(bytes)
  }
}

/// Verify a block hash from serialized bytes.
///
/// This verifies only the block hash against the beacon chain data.
pub fn verify_block_hash_from_bytes(bytes: &[u8]) -> Result<(), VerifyError> {
  let input =
    VerifyBlockHashInput::from_bytes(bytes).map_err(|_| VerifyError::DeserializationError)?;

  verify_block_hash(
    input.block_hash,
    &input.block_response,
    &input.header_response,
    input.network,
  )
}

/// Input for zkVM header verification.
///
/// This struct contains all the data needed to verify a Linea block header
/// inside a zkVM. It can be serialized to bytes for passing into the zkVM
/// and deserialized inside for verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyInput {
  /// The execution layer header.
  pub input_header: Header,
  /// The beacon block response.
  pub block_response: BlockResponse,
  /// The beacon header response.
  pub header_response: HeaderResponse,
  /// The network.
  pub network: LineaNetwork,
}

impl VerifyInput {
  /// Create a new verification input from the component parts.
  pub fn new(
    input_header: Header,
    block_response: BlockResponse,
    header_response: HeaderResponse,
    network: LineaNetwork,
  ) -> Self {
    Self {
      input_header,
      block_response,
      header_response,
      network,
    }
  }

  /// Serialize this input to bytes for passing into zkVM.
  pub fn to_bytes(&self) -> Result<Vec<u8>, VerifyError> {
    bincode::serialize(self).map_err(|_| VerifyError::SerializationError)
  }

  /// Deserialize input from bytes inside zkVM.
  pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
    bincode::deserialize(bytes)
  }

  /// Convert to a block hash input (for hash-only verification).
  pub fn to_block_hash_input(&self) -> VerifyBlockHashInput {
    VerifyBlockHashInput {
      block_hash: self.input_header.hash_slow(),
      block_response: self.block_response.clone(),
      header_response: self.header_response.clone(),
      network: self.network,
    }
  }
}

/// Verify a header from serialized bytes.
///
/// This is the main entry point for zkVM verification. It deserializes
/// the input and performs all verification checks including Linea bug detection.
pub fn verify_from_bytes(bytes: &[u8]) -> Result<(), VerifyError> {
  let input = VerifyInput::from_bytes(bytes).map_err(|_| VerifyError::DeserializationError)?;

  verify_header(
    &input.input_header,
    &input.block_response,
    &input.header_response,
    input.network,
  )
}
