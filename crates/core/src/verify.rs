//! Header verification logic for Linea V4 proofs.
//!
//! This module provides verification of Linea block headers against beacon chain data.
//! It validates the cryptographic chain of trust from the sequencer's ECDSA signature
//! down to the user's block hash or full header.

use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloy_consensus::Header;
use alloy_primitives::{Address, B256, Signature};
use alloy_rlp::Encodable;

use crate::beacon::{Attestation, BlockResponse, ExecutionPayload, HeaderResponse};
use crate::constants::{LineaNetwork, get_genesis_beacon_root, get_sequencer_address};
use crate::error::VerifyError;
use crate::{compute_beacon_block_root, compute_body_hash};

/// Verifies that a block hash is authentic by checking it against the Linea beacon chain data.
///
/// This is the simpler entry point when you only have a block hash to verify.
/// It proves that the given block hash was signed by the Linea sequencer.
pub fn verify_block_hash(
  block_hash: B256,
  block_response: &BlockResponse,
  header_response: &HeaderResponse,
  network: LineaNetwork,
) -> Result<(), VerifyError> {
  // Validate beacon data and get trusted execution payload
  let execution_payload = verify_beacon_data(block_response, header_response, network)?;

  // Verify user's block hash matches the trusted execution payload
  if execution_payload.block_hash != block_hash {
    return Err(VerifyError::BlockHashMismatch {
      expected: block_hash,
      got: execution_payload.block_hash,
    });
  }

  Ok(())
}

/// Verifies that a full execution header is authentic by checking it against the Linea beacon chain data.
///
/// This entry point validates a complete header and performs additional consistency checks.
/// If the block hash matches but fields differ, it indicates a bug in Linea (LineaBugDetected).
pub fn verify_header(
  input_header: &Header,
  block_response: &BlockResponse,
  header_response: &HeaderResponse,
  network: LineaNetwork,
) -> Result<(), VerifyError> {
  // Validate beacon data and get trusted execution payload
  let execution_payload = verify_beacon_data(block_response, header_response, network)?;

  // Compute block hash from input header
  let input_block_hash = input_header.hash_slow();

  // Verify block hash matches
  if execution_payload.block_hash != input_block_hash {
    return Err(VerifyError::BlockHashMismatch {
      expected: input_block_hash,
      got: execution_payload.block_hash,
    });
  }

  // Block hash matches - now verify all fields for Linea bug detection
  verify_execution_payload_fields(execution_payload, input_header)?;

  Ok(())
}

/// Core beacon data validation.
///
/// This function validates the beacon chain data and returns the trusted ExecutionPayload.
/// After this function succeeds, the ExecutionPayload can be trusted.
///
/// Verification steps:
/// 0. RPC sanity checks (version, signature, proposer)
/// 1. Slot validation (not zero)
/// 2. Previous attestation validation (for slots > 1)
/// 3. Body hash validation
/// 4. Header response validation (slot, parent_root, timestamp)
/// 5. Current attestation validation
/// 6. Beacon block root validation
fn verify_beacon_data<'a>(
  block_response: &'a BlockResponse,
  header_response: &HeaderResponse,
  network: LineaNetwork,
) -> Result<&'a ExecutionPayload, VerifyError> {
  let sequencer = get_sequencer_address(network);
  let beacon_block = &block_response.data.message;
  let beacon_header = &header_response.data.message;
  let target_slot = beacon_block.slot;

  // =========================================================================
  // Step 0: RPC sanity checks
  // =========================================================================

  // Verify beacon version is "maru"
  if block_response.version != "maru" {
    return Err(VerifyError::UnsupportedVersion(
      block_response.version.clone(),
    ));
  }

  // Verify beacon block signature is empty (Linea-specific)
  if !block_response.data.signature.is_empty() {
    return Err(VerifyError::BeaconSignatureNotEmpty);
  }

  // Verify proposer is the expected sequencer
  let proposer: Address = beacon_block
    .proposer_index
    .parse()
    .map_err(|_| VerifyError::InvalidProposerAddress(beacon_block.proposer_index.clone()))?;
  if proposer != sequencer {
    return Err(VerifyError::InvalidProposer {
      expected: sequencer,
      got: proposer,
    });
  }

  // =========================================================================
  // Step 1: Slot validation
  // =========================================================================

  if target_slot == 0 {
    return Err(VerifyError::InvalidSlotZero);
  }

  // =========================================================================
  // Step 2: Find previous attestation (for slots > 1)
  // =========================================================================

  let prev_slot = target_slot.saturating_sub(1);
  let prev_attestation = if target_slot > 1 {
    Some(
      beacon_block
        .body
        .attestations
        .iter()
        .find(|att| att.data.slot == prev_slot)
        .ok_or(VerifyError::NoPrevAttestation(prev_slot))?,
    )
  } else {
    None
  };

  // =========================================================================
  // Step 3: Validate previous attestation signature
  // =========================================================================

  if let Some(attestation) = prev_attestation {
    verify_attestation_signature(attestation, sequencer, AttestationType::Previous)?;
  }

  // =========================================================================
  // Step 4: Compute and verify body hash
  // =========================================================================

  let computed_body_hash =
    compute_body_hash(prev_attestation, &beacon_block.body.execution_payload);

  if computed_body_hash != beacon_header.body_root {
    return Err(VerifyError::BodyHashMismatch {
      computed: computed_body_hash,
      expected: beacon_header.body_root,
    });
  }

  // =========================================================================
  // Step 5: Validate beacon header slot
  // =========================================================================

  if beacon_header.slot != target_slot {
    return Err(VerifyError::BeaconSlotMismatch {
      expected: target_slot,
      got: beacon_header.slot,
    });
  }

  // =========================================================================
  // Step 6: Validate beacon header parent_root
  // =========================================================================

  let expected_parent_root = if target_slot == 1 {
    // For slot 1, parent_root should be the genesis beacon root
    get_genesis_beacon_root(network)
  } else {
    // For slots > 1, parent_root should match the previous attestation's beacon_block_root
    prev_attestation
      .expect("prev_attestation should exist for slots > 1")
      .data
      .beacon_block_root
  };

  if beacon_header.parent_root != expected_parent_root {
    return Err(VerifyError::BeaconParentRootMismatch {
      expected: expected_parent_root,
      got: beacon_header.parent_root,
    });
  }

  // =========================================================================
  // Step 7: Validate beacon header timestamp matches execution payload
  // =========================================================================

  if beacon_header.timestamp != beacon_block.body.execution_payload.timestamp {
    return Err(VerifyError::BeaconTimestampMismatch {
      expected: beacon_block.body.execution_payload.timestamp,
      got: beacon_header.timestamp,
    });
  }

  // =========================================================================
  // Step 8: Find current attestation
  // =========================================================================

  let current_attestation = beacon_block
    .body
    .attestations
    .iter()
    .find(|att| att.data.slot == target_slot)
    .ok_or(VerifyError::NoAttestationForSlot(target_slot))?;

  // =========================================================================
  // Step 9: Validate current attestation signature
  // =========================================================================

  verify_attestation_signature(current_attestation, sequencer, AttestationType::Current)?;

  // =========================================================================
  // Step 10: Compute and verify beacon block root
  // =========================================================================

  let computed_beacon_root = compute_beacon_block_root(beacon_header);
  let attested_beacon_root = current_attestation.data.beacon_block_root;

  if computed_beacon_root != attested_beacon_root {
    return Err(VerifyError::BeaconBlockRootMismatch {
      computed: computed_beacon_root,
      expected: attested_beacon_root,
    });
  }

  // =========================================================================
  // Beacon data is now fully validated - ExecutionPayload can be trusted
  // =========================================================================

  Ok(&beacon_block.body.execution_payload)
}

/// Which attestation is being verified (for error reporting).
#[derive(Debug, Clone, Copy)]
enum AttestationType {
  Current,
  Previous,
}

/// Verifies an attestation signature was made by the expected sequencer.
fn verify_attestation_signature(
  attestation: &Attestation,
  expected_sequencer: Address,
  attestation_type: AttestationType,
) -> Result<(), VerifyError> {
  let signature = Signature::try_from(attestation.signature.as_ref())
    .map_err(|_| VerifyError::InvalidSignature)?;

  let recovered_address = signature
    .recover_address_from_prehash(&attestation.data.beacon_block_root)
    .map_err(|_| VerifyError::InvalidSignature)?;

  if recovered_address != expected_sequencer {
    return Err(match attestation_type {
      AttestationType::Current => VerifyError::SignerMismatch {
        expected: expected_sequencer,
        recovered: recovered_address,
      },
      AttestationType::Previous => VerifyError::PrevAttestationSignerMismatch {
        expected: expected_sequencer,
        recovered: recovered_address,
      },
    });
  }

  Ok(())
}

/// Verifies all execution payload fields match the input header fields.
///
/// This is called after block hash verification succeeds. If any field mismatches,
/// it indicates a bug in Linea (since the hash matched but a field differs).
fn verify_execution_payload_fields(
  execution_payload: &ExecutionPayload,
  input_header: &Header,
) -> Result<(), VerifyError> {
  // Block number
  if execution_payload.block_number != input_header.number {
    return Err(VerifyError::LineaBugDetected(format!(
      "block_number: expected {}, got {}",
      input_header.number, execution_payload.block_number
    )));
  }

  // Parent hash
  if execution_payload.parent_hash != input_header.parent_hash {
    return Err(VerifyError::LineaBugDetected(format!(
      "parent_hash: expected {}, got {}",
      input_header.parent_hash, execution_payload.parent_hash
    )));
  }

  // State root
  if execution_payload.state_root != input_header.state_root {
    return Err(VerifyError::LineaBugDetected(format!(
      "state_root: expected {}, got {}",
      input_header.state_root, execution_payload.state_root
    )));
  }

  // Receipts root
  if execution_payload.receipts_root != input_header.receipts_root {
    return Err(VerifyError::LineaBugDetected(format!(
      "receipts_root: expected {}, got {}",
      input_header.receipts_root, execution_payload.receipts_root
    )));
  }

  // Fee recipient (beneficiary)
  if execution_payload.fee_recipient != input_header.beneficiary {
    return Err(VerifyError::LineaBugDetected(format!(
      "fee_recipient: expected {}, got {}",
      input_header.beneficiary, execution_payload.fee_recipient
    )));
  }

  // Logs bloom
  if execution_payload.logs_bloom != input_header.logs_bloom {
    return Err(VerifyError::LineaBugDetected(
      "logs_bloom mismatch".to_string(),
    ));
  }

  // Timestamp
  if execution_payload.timestamp != input_header.timestamp {
    return Err(VerifyError::LineaBugDetected(format!(
      "timestamp: expected {}, got {}",
      input_header.timestamp, execution_payload.timestamp
    )));
  }

  // Gas limit
  if execution_payload.gas_limit != input_header.gas_limit {
    return Err(VerifyError::LineaBugDetected(format!(
      "gas_limit: expected {}, got {}",
      input_header.gas_limit, execution_payload.gas_limit
    )));
  }

  // Gas used
  if execution_payload.gas_used != input_header.gas_used {
    return Err(VerifyError::LineaBugDetected(format!(
      "gas_used: expected {}, got {}",
      input_header.gas_used, execution_payload.gas_used
    )));
  }

  // Base fee per gas
  let expected_base_fee = input_header.base_fee_per_gas.ok_or_else(|| {
    VerifyError::LineaBugDetected("missing base_fee_per_gas in header".to_string())
  })?;
  if execution_payload.base_fee_per_gas != expected_base_fee {
    return Err(VerifyError::LineaBugDetected(format!(
      "base_fee_per_gas: expected {}, got {}",
      expected_base_fee, execution_payload.base_fee_per_gas
    )));
  }

  // Extra data
  if execution_payload.extra_data.as_ref() != input_header.extra_data.as_ref() {
    return Err(VerifyError::LineaBugDetected(
      "extra_data mismatch".to_string(),
    ));
  }

  // Prev randao (maps to mix_hash in header)
  if execution_payload.prev_randao != input_header.mix_hash {
    return Err(VerifyError::LineaBugDetected(format!(
      "prev_randao: expected {}, got {}",
      input_header.mix_hash, execution_payload.prev_randao
    )));
  }

  // Transactions root
  let computed_transactions_root = compute_transactions_root(&execution_payload.transactions);
  if computed_transactions_root != input_header.transactions_root {
    return Err(VerifyError::LineaBugDetected(format!(
      "transactions_root: expected {}, got {}",
      input_header.transactions_root, computed_transactions_root
    )));
  }

  Ok(())
}

/// Computes the transactions root from a list of RLP-encoded transactions.
///
/// The transactions root is computed by building a Merkle Patricia Trie where:
/// - Keys: RLP-encoded transaction indices (0, 1, 2, ...)
/// - Values: RLP-encoded transactions
fn compute_transactions_root(transactions: &[alloy_primitives::Bytes]) -> B256 {
  use alloc::collections::BTreeMap;

  // Build key-value pairs with RLP-encoded indices as keys
  let mut pairs: BTreeMap<Vec<u8>, &[u8]> = BTreeMap::new();
  for (idx, tx) in transactions.iter().enumerate() {
    let mut key_buf = Vec::new();
    idx.encode(&mut key_buf);
    pairs.insert(key_buf, tx.as_ref());
  }

  // HashBuilder requires keys in sorted order
  let mut hb = alloy_trie::HashBuilder::default();
  for (key, value) in pairs {
    hb.add_leaf(alloy_trie::Nibbles::unpack(&key), value);
  }

  hb.root()
}

// Tests for genesis beacon root constants are in the host crate
// since they require network access to fetch slot 0 data.
