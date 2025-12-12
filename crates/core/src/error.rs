//! Error types for header verification.

use alloc::string::String;
use alloy_primitives::{Address, B256};
use core::fmt;

/// Error type for header verification failures.
#[derive(Debug)]
pub enum VerifyError {
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
  DeserializationError,
}

#[cfg(feature = "std")]
impl std::error::Error for VerifyError {}

impl fmt::Display for VerifyError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      VerifyError::UnsupportedVersion(version) => {
        write!(f, "Unsupported beacon version: {}, expected 'maru'", version)
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
      VerifyError::DeserializationError => {
        write!(f, "Failed to deserialize input")
      }
    }
  }
}
