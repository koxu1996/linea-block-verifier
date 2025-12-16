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

//! Error types for header verification.

use alloc::string::String;
use alloy_primitives::{Address, B256};
use core::fmt;

/// Error type for header verification failures.
#[derive(Debug)]
pub enum VerifyError {
  UnsupportedChainId(u64),
  BlockAtOrBeforeActivation {
    block_number: u64,
    activation_block: u64,
  },
  InvalidSlotZero,
  InvalidProposerAddress(String),
  UnsupportedVersion(String),
  BeaconSignatureNotEmpty,
  InvalidProposer {
    expected: Address,
    got: Address,
  },
  NoAttestationForSlot(u64),
  BeaconBlockRootMismatch {
    computed: B256,
    expected: B256,
  },
  BodyHashMismatch {
    computed: B256,
    expected: B256,
  },
  InvalidSignature,
  SignerMismatch {
    expected: Address,
    recovered: Address,
  },
  BlockNumberMismatch {
    expected: u64,
    got: u64,
  },
  BlockHashMismatch {
    expected: B256,
    got: B256,
  },
  ParentHashMismatch {
    expected: B256,
    got: B256,
  },
  StateRootMismatch {
    expected: B256,
    got: B256,
  },
  ReceiptsRootMismatch {
    expected: B256,
    got: B256,
  },
  FeeRecipientMismatch {
    expected: Address,
    got: Address,
  },
  LogsBloomMismatch,
  TimestampMismatch {
    expected: u64,
    got: u64,
  },
  GasLimitMismatch {
    expected: u64,
    got: u64,
  },
  GasUsedMismatch {
    expected: u64,
    got: u64,
  },
  BaseFeePerGasMismatch {
    expected: u64,
    got: u64,
  },
  MissingBaseFeePerGas,
  ExtraDataMismatch,
  PrevRandaoMismatch {
    expected: B256,
    got: B256,
  },
  TransactionsRootMismatch {
    expected: B256,
    got: B256,
  },
  PrevAttestationSignerMismatch {
    expected: Address,
    recovered: Address,
  },
  PrevAttestationBlockRootMismatch {
    expected: B256,
    got: B256,
  },
  NoPrevAttestation(u64),
  /// Beacon header slot does not match block response slot.
  BeaconSlotMismatch {
    expected: u64,
    got: u64,
  },
  /// Beacon header timestamp does not match execution payload timestamp.
  BeaconTimestampMismatch {
    expected: u64,
    got: u64,
  },
  /// Beacon header parent_root does not match expected value.
  BeaconParentRootMismatch {
    expected: B256,
    got: B256,
  },
  /// Block hash matches but a field differs - indicates a bug in Linea.
  LineaBugDetected(String),
  DeserializationError,
  SerializationError,
}

#[cfg(feature = "std")]
impl std::error::Error for VerifyError {}

impl fmt::Display for VerifyError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      VerifyError::UnsupportedChainId(chain_id) => {
        write!(f, "Unsupported chain ID: {}", chain_id)
      }
      VerifyError::BlockAtOrBeforeActivation {
        block_number,
        activation_block,
      } => {
        write!(
          f,
          "Block number {} is at or before activation block {}",
          block_number, activation_block
        )
      }
      VerifyError::InvalidSlotZero => {
        write!(
          f,
          "Slot 0 is invalid: it represents the activation block which has no beacon data"
        )
      }
      VerifyError::InvalidProposerAddress(addr) => {
        write!(f, "Invalid proposer address in beacon block: {}", addr)
      }
      VerifyError::UnsupportedVersion(version) => {
        write!(
          f,
          "Unsupported beacon version: {}, expected 'maru'",
          version
        )
      }
      VerifyError::BeaconSignatureNotEmpty => {
        write!(f, "Expected empty beacon block signature")
      }
      VerifyError::InvalidProposer { expected, got } => {
        write!(f, "Invalid proposer: expected {}, got {}", expected, got)
      }
      VerifyError::NoAttestationForSlot(slot) => {
        write!(f, "No attestation found for slot {}", slot)
      }
      VerifyError::BeaconBlockRootMismatch { computed, expected } => {
        write!(
          f,
          "Beacon block root mismatch: computed {}, expected {}",
          computed, expected
        )
      }
      VerifyError::BodyHashMismatch { computed, expected } => {
        write!(
          f,
          "Body hash mismatch: computed {}, expected {}",
          computed, expected
        )
      }
      VerifyError::InvalidSignature => {
        write!(f, "Invalid signature")
      }
      VerifyError::SignerMismatch {
        expected,
        recovered,
      } => {
        write!(
          f,
          "Signer mismatch: expected {}, recovered {}",
          expected, recovered
        )
      }
      VerifyError::BlockNumberMismatch { expected, got } => {
        write!(
          f,
          "Block number mismatch: expected {}, got {}",
          expected, got
        )
      }
      VerifyError::BlockHashMismatch { expected, got } => {
        write!(f, "Block hash mismatch: expected {}, got {}", expected, got)
      }
      VerifyError::ParentHashMismatch { expected, got } => {
        write!(
          f,
          "Parent hash mismatch: expected {}, got {}",
          expected, got
        )
      }
      VerifyError::StateRootMismatch { expected, got } => {
        write!(f, "State root mismatch: expected {}, got {}", expected, got)
      }
      VerifyError::ReceiptsRootMismatch { expected, got } => {
        write!(
          f,
          "Receipts root mismatch: expected {}, got {}",
          expected, got
        )
      }
      VerifyError::FeeRecipientMismatch { expected, got } => {
        write!(
          f,
          "Fee recipient mismatch: expected {}, got {}",
          expected, got
        )
      }
      VerifyError::LogsBloomMismatch => {
        write!(f, "Logs bloom mismatch")
      }
      VerifyError::TimestampMismatch { expected, got } => {
        write!(f, "Timestamp mismatch: expected {}, got {}", expected, got)
      }
      VerifyError::GasLimitMismatch { expected, got } => {
        write!(f, "Gas limit mismatch: expected {}, got {}", expected, got)
      }
      VerifyError::GasUsedMismatch { expected, got } => {
        write!(f, "Gas used mismatch: expected {}, got {}", expected, got)
      }
      VerifyError::BaseFeePerGasMismatch { expected, got } => {
        write!(
          f,
          "Base fee per gas mismatch: expected {}, got {}",
          expected, got
        )
      }
      VerifyError::MissingBaseFeePerGas => {
        write!(f, "Missing base_fee_per_gas in header")
      }
      VerifyError::ExtraDataMismatch => {
        write!(f, "Extra data mismatch")
      }
      VerifyError::PrevRandaoMismatch { expected, got } => {
        write!(
          f,
          "Prev randao mismatch: expected {}, got {}",
          expected, got
        )
      }
      VerifyError::TransactionsRootMismatch { expected, got } => {
        write!(
          f,
          "Transactions root mismatch: expected {}, got {}",
          expected, got
        )
      }
      VerifyError::PrevAttestationSignerMismatch {
        expected,
        recovered,
      } => {
        write!(
          f,
          "Previous attestation signer mismatch: expected {}, recovered {}",
          expected, recovered
        )
      }
      VerifyError::PrevAttestationBlockRootMismatch { expected, got } => {
        write!(
          f,
          "Previous attestation block root mismatch: expected {}, got {}",
          expected, got
        )
      }
      VerifyError::NoPrevAttestation(slot) => {
        write!(f, "No previous attestation found for slot {}", slot)
      }
      VerifyError::BeaconSlotMismatch { expected, got } => {
        write!(
          f,
          "Beacon header slot mismatch: expected {}, got {}",
          expected, got
        )
      }
      VerifyError::BeaconTimestampMismatch { expected, got } => {
        write!(
          f,
          "Beacon header timestamp mismatch: expected {}, got {}",
          expected, got
        )
      }
      VerifyError::BeaconParentRootMismatch { expected, got } => {
        write!(
          f,
          "Beacon header parent_root mismatch: expected {}, got {}",
          expected, got
        )
      }
      VerifyError::LineaBugDetected(details) => {
        write!(
          f,
          "Linea bug detected: block hash matches but field differs: {}",
          details
        )
      }
      VerifyError::DeserializationError => {
        write!(f, "Failed to deserialize input")
      }
      VerifyError::SerializationError => {
        write!(f, "Failed to serialize input")
      }
    }
  }
}
