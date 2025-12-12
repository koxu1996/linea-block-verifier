//! zkVM input/output serialization utilities.
//!
//! This module provides functionality to serialize verification inputs
//! for use in zkVM environments and deserialize them for verification.
//!
//! Only available when the `std` feature is enabled.

use alloy_consensus::Header;
use serde::{Deserialize, Serialize};

use crate::beacon::{BlockResponse, HeaderResponse};
use crate::constants::LineaNetwork;
use crate::error::VerifyError;
use crate::verify::verify_header;

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
  pub fn to_bytes(&self) -> Vec<u8> {
    bincode::serialize(self).expect("serialization failed")
  }

  /// Deserialize input from bytes inside zkVM.
  pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
    bincode::deserialize(bytes)
  }
}

/// Verify a header from serialized bytes.
///
/// This is the main entry point for zkVM verification. It deserializes
/// the input and performs all verification checks.
pub fn verify_from_bytes(bytes: &[u8]) -> Result<(), VerifyError> {
  let input = VerifyInput::from_bytes(bytes).map_err(|_| VerifyError::DeserializationError)?;

  verify_header(
    &input.input_header,
    &input.block_response,
    &input.header_response,
    input.network,
  )
}
