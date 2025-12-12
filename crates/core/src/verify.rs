//! Header verification logic.

use alloc::vec::Vec;
use alloy_consensus::Header;
use alloy_primitives::{Address, Signature};

use crate::beacon::{BlockResponse, HeaderResponse};
use crate::constants::{LineaNetwork, get_sequencer_address};
use crate::error::VerifyError;
use crate::{compute_beacon_block_root, compute_body_hash};

/// Verifies that an execution layer header is authentic by checking it against
/// the Linea beacon chain data.
///
/// This function performs the following verifications:
/// 1. Beacon block signature is empty (Linea-specific)
/// 2. Proposer matches expected sequencer address
/// 3. Beacon block root is correctly computed and matches attestation
/// 4. Body hash is correctly computed and matches header
/// 5. Attestation ECDSA signature is valid and signed by sequencer
/// 6. All 12 execution header fields match the beacon execution payload
pub fn verify_header(
  input_header: &Header,
  block_response: &BlockResponse,
  header_response: &HeaderResponse,
  network: LineaNetwork,
) -> Result<(), VerifyError> {
  let expected_sequencer = get_sequencer_address(network);
  let input_block_hash = input_header.hash_slow();

  let beacon_block = &block_response.data.message;
  let beacon_block_signature = &block_response.data.signature;
  let beacon_header = &header_response.data.message;
  let beacon_slot = beacon_block.slot;

  // Check 1: Verify beacon block signature is empty (not used by Linea).
  if !beacon_block_signature.is_empty() {
    return Err(VerifyError::BeaconSignatureNotEmpty);
  }

  // Check 2: Verify proposer is the expected Linea sequencer.
  let proposer: Address = beacon_block
    .proposer_index
    .parse()
    .expect("Invalid proposer address in beacon block");
  if proposer != expected_sequencer {
    return Err(VerifyError::InvalidProposer {
      expected: expected_sequencer,
      got: proposer,
    });
  }

  // Check 3: Find attestation for current slot.
  let attestations = &beacon_block.body.attestations;
  let current_attestation = attestations
    .iter()
    .find(|att| att.data.slot == beacon_slot)
    .ok_or(VerifyError::NoAttestationForSlot(beacon_slot))?;

  // Check 4: Compute and verify beacon block root.
  let computed_root = compute_beacon_block_root(beacon_header);
  let beacon_block_root = current_attestation.data.beacon_block_root;

  if computed_root != beacon_block_root {
    return Err(VerifyError::BeaconBlockRootMismatch {
      computed: computed_root,
      expected: beacon_block_root,
    });
  }

  // Check 5: Compute and verify the body hash.
  let prev_attestations: Vec<_> = beacon_block
    .body
    .attestations
    .iter()
    .filter(|att| att.data.slot < beacon_slot)
    .collect();

  let computed_body_hash =
    compute_body_hash(&prev_attestations, &beacon_block.body.execution_payload);

  let expected_body_root = beacon_header.body_root;

  if computed_body_hash != expected_body_root {
    return Err(VerifyError::BodyHashMismatch {
      computed: computed_body_hash,
      expected: expected_body_root,
    });
  }

  // Check 6: Verify attestation ECDSA signature.
  let signature = Signature::try_from(current_attestation.signature.as_ref())
    .map_err(|_| VerifyError::InvalidSignature)?;

  let recovered_address = signature
    .recover_address_from_prehash(&beacon_block_root)
    .map_err(|_| VerifyError::InvalidSignature)?;

  if recovered_address != expected_sequencer {
    return Err(VerifyError::SignerMismatch {
      expected: expected_sequencer,
      recovered: recovered_address,
    });
  }

  // Check 7-18: Verify all execution header fields match the beacon execution payload.
  let execution_payload = &beacon_block.body.execution_payload;

  // Block number
  if execution_payload.block_number != input_header.number {
    return Err(VerifyError::BlockNumberMismatch {
      expected: input_header.number,
      got: execution_payload.block_number,
    });
  }

  // Block hash
  if execution_payload.block_hash != input_block_hash {
    return Err(VerifyError::BlockHashMismatch {
      expected: input_block_hash,
      got: execution_payload.block_hash,
    });
  }

  // Parent hash
  if execution_payload.parent_hash != input_header.parent_hash {
    return Err(VerifyError::ParentHashMismatch {
      expected: input_header.parent_hash,
      got: execution_payload.parent_hash,
    });
  }

  // State root
  if execution_payload.state_root != input_header.state_root {
    return Err(VerifyError::StateRootMismatch {
      expected: input_header.state_root,
      got: execution_payload.state_root,
    });
  }

  // Receipts root
  if execution_payload.receipts_root != input_header.receipts_root {
    return Err(VerifyError::ReceiptsRootMismatch {
      expected: input_header.receipts_root,
      got: execution_payload.receipts_root,
    });
  }

  // Fee recipient (beneficiary)
  if execution_payload.fee_recipient != input_header.beneficiary {
    return Err(VerifyError::FeeRecipientMismatch {
      expected: input_header.beneficiary,
      got: execution_payload.fee_recipient,
    });
  }

  // Logs bloom
  if execution_payload.logs_bloom != input_header.logs_bloom {
    return Err(VerifyError::LogsBloomMismatch);
  }

  // Timestamp
  if execution_payload.timestamp != input_header.timestamp {
    return Err(VerifyError::TimestampMismatch {
      expected: input_header.timestamp,
      got: execution_payload.timestamp,
    });
  }

  // Gas limit
  if execution_payload.gas_limit != input_header.gas_limit {
    return Err(VerifyError::GasLimitMismatch {
      expected: input_header.gas_limit,
      got: execution_payload.gas_limit,
    });
  }

  // Gas used
  if execution_payload.gas_used != input_header.gas_used {
    return Err(VerifyError::GasUsedMismatch {
      expected: input_header.gas_used,
      got: execution_payload.gas_used,
    });
  }

  // Base fee per gas
  let expected_base_fee = input_header
    .base_fee_per_gas
    .ok_or(VerifyError::MissingBaseFeePerGas)?;
  if execution_payload.base_fee_per_gas != expected_base_fee {
    return Err(VerifyError::BaseFeePerGasMismatch {
      expected: expected_base_fee,
      got: execution_payload.base_fee_per_gas,
    });
  }

  // Extra data
  if execution_payload.extra_data.as_ref() != input_header.extra_data.as_ref() {
    return Err(VerifyError::ExtraDataMismatch);
  }

  Ok(())
}
