//! zkVM input/output serialization utilities.
//!
//! This module provides functionality to serialize verification inputs
//! for use in zkVM environments and deserialize them for verification.
//!
//! Only available when the `std` feature is enabled.

use alloy_consensus::Header;
use alloy_primitives::B256;
use serde::{Deserialize, Serialize};

use crate::constants::LineaNetwork;
use crate::error::VerifyError;
use crate::types::BeaconData;
use crate::verify::{verify_block_hash, verify_header};

/// User-provided input for verification.
///
/// This represents what the user wants to verify - either a block hash
/// or a full header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserInput {
  /// Verify only the block hash (faster, smaller input).
  BlockHash(B256),
  /// Verify the full header with Linea bug detection.
  Header(Header),
}

/// Complete zkVM verification input.
///
/// This struct contains all the data needed to verify a Linea block
/// inside a zkVM. It separates user-provided input from auxiliary
/// beacon chain data fetched by the host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyInput {
  /// User-provided input (block hash or full header).
  pub user_input: UserInput,
  /// Auxiliary beacon chain data fetched by the host.
  pub beacon_data: BeaconData,
  /// The network.
  pub network: LineaNetwork,
}

impl VerifyInput {
  /// Create a new verification input for a block hash.
  pub fn new_block_hash(block_hash: B256, beacon_data: BeaconData, network: LineaNetwork) -> Self {
    Self {
      user_input: UserInput::BlockHash(block_hash),
      beacon_data,
      network,
    }
  }

  /// Create a new verification input for a full header.
  pub fn new_header(header: Header, beacon_data: BeaconData, network: LineaNetwork) -> Self {
    Self {
      user_input: UserInput::Header(header),
      beacon_data,
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

/// Verify input from serialized bytes.
///
/// This is the main entry point for zkVM verification. It deserializes
/// the input and performs verification based on the input type:
/// - `UserInput::BlockHash`: Verifies only the block hash
/// - `UserInput::Header`: Verifies the full header with Linea bug detection
///
/// Returns the trusted block hash on success.
pub fn verify_from_bytes(bytes: &[u8]) -> Result<B256, VerifyError> {
  let input = VerifyInput::from_bytes(bytes).map_err(|_| VerifyError::DeserializationError)?;

  match &input.user_input {
    UserInput::BlockHash(block_hash) => {
      verify_block_hash(
        *block_hash,
        &input.beacon_data.block_response,
        &input.beacon_data.header_response,
        input.network,
      )?;
      Ok(*block_hash)
    }
    UserInput::Header(header) => verify_header(
      header,
      &input.beacon_data.block_response,
      &input.beacon_data.header_response,
      input.network,
    ),
  }
}
