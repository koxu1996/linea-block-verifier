# Security Model & Verification Logic

This document describes the cryptographic verification flow and security model for the Linea Block Verifier.

## Table of Contents

1. [Overview](#overview)
2. [Trust Model](#trust-model)
3. [Cryptographic Chain of Trust](#cryptographic-chain-of-trust)
4. [Verification Steps](#verification-steps)
5. [Attack Surface Analysis](#attack-surface-analysis)
6. [Verification Modes](#verification-modes)

---

## Overview

The verifier proves that a Linea block hash (or full header) was signed by the Linea sequencer through the Maru consensus beacon chain. The core insight is:

> **If we can prove that the Linea sequencer signed a beacon_block_root, and we can cryptographically link that root to a specific block hash, then the block hash is authentic.**

The verification builds a chain of cryptographic commitments from the sequencer's ECDSA signature down to the user's block hash.

---

## Trust Model

### Hardcoded Constants (Trusted)

Only these values are hardcoded and must be trusted:

| Constant | Mainnet | Sepolia | Purpose |
|----------|---------|---------|---------|
| Sequencer Address | `0x9f31...9bc` | `0x20e2...13f` | ECDSA signature verification |
| Activation Block | 24,787,631 | 18,665,903 | Slot calculation: `slot = block_number - activation_block` |
| Genesis Beacon Root | `0xc05c...892` | `0x551a...8a2` | Parent root validation for slot 1 |

### Untrusted Inputs

Everything else comes from untrusted sources (RPC responses) and is validated cryptographically:

- `BlockResponse` - Contains beacon block with attestations and execution payload
- `HeaderResponse` - Contains beacon header
- User's block hash or header - What we're verifying

The RPC can only affect **availability** (refuse to respond), not **integrity** (cannot forge valid data).

---

## Cryptographic Chain of Trust

The verification establishes trust through this chain:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  STEP 9: CURRENT ATTESTATION SIGNATURE                                      │
│                                                                             │
│  attestation.signature ──► recover_address() ══► must equal SEQUENCER      │
│  attestation.data.beacon_block_root ◄───────────────────────────────────────│
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼ must equal (Step 10)
┌─────────────────────────────────────────────────────────────────────────────┐
│  STEP 10: BEACON BLOCK ROOT                                                 │
│                                                                             │
│  computed = keccak256(rlp([slot, round, timestamp, [proposer],              │
│                            parent_root, state_root, body_root]))            │
│                                    │                                        │
│  beacon_header.body_root ◄─────────│                                        │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼ must equal (Step 4)
┌─────────────────────────────────────────────────────────────────────────────┐
│  STEP 4: BODY HASH                                                          │
│                                                                             │
│  computed = keccak256(rlp([[prev_attestation.signature],                    │
│                             execution_payload]))                            │
│                                    │                                        │
│  execution_payload.block_hash ◄────│                                        │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼ must equal (Step 11)
┌─────────────────────────────────────────────────────────────────────────────┐
│  STEP 11: USER INPUT                                                        │
│                                                                             │
│  user's block_hash (or computed from user's header via hash_slow())         │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Key insight**: Each arrow represents a cryptographic commitment. If any link is broken, the verification fails. Only the Linea sequencer can produce a valid chain.

---

## Verification Steps

The verification performs 11 steps (numbered 0-10 in code, plus final user input check):

### Step 0: RPC Sanity Checks

Before cryptographic validation, basic sanity checks on RPC response:

| Check | Validation | Error |
|-------|------------|-------|
| Version | `block_response.version == "maru"` | `UnsupportedVersion` |
| Block Signature | `block_response.data.signature.is_empty()` | `BeaconSignatureNotEmpty` |
| Proposer | `beacon_block.proposer_index == sequencer` | `InvalidProposer` |

**Why?** Ensures we're talking to a Maru consensus node and the proposer is the expected sequencer.

### Step 1: Slot Validation

```rust
if target_slot == 0 {
    return Err(InvalidSlotZero);
}
```

**Why?** Slot 0 is the activation block itself, which has no beacon data to verify.

### Step 2: Find Previous Attestation

For slots > 1, find the attestation for the previous slot:

```rust
let prev_attestation = beacon_block.body.attestations
    .find(|att| att.data.slot == target_slot - 1);
```

**Why?** The previous attestation's signature is included in the body hash (chain continuity).

### Step 3: Previous Attestation Signature

```rust
verify_attestation_signature(prev_attestation, sequencer, Previous)?;
```

Recovers the signer from the ECDSA signature and verifies it matches the sequencer.

**Why?** Ensures chain continuity - the previous block was also signed by the sequencer.

### Step 4: Body Hash Validation

```rust
let computed = compute_body_hash(prev_attestation, &execution_payload);
if computed != beacon_header.body_root {
    return Err(BodyHashMismatch);
}
```

Body hash computation:
```
body = rlp([[prev_attestation.signature], execution_payload])
body_root = keccak256(body)
```

**Why?** Links the execution payload (containing block_hash) to the beacon header.

### Step 5: Beacon Header Slot

```rust
if beacon_header.slot != target_slot {
    return Err(BeaconSlotMismatch);
}
```

**Why?** Ensures the header corresponds to the correct slot.

### Step 6: Beacon Header Parent Root

```rust
let expected_parent = if target_slot == 1 {
    get_genesis_beacon_root(network)  // Hardcoded constant
} else {
    prev_attestation.data.beacon_block_root
};
if beacon_header.parent_root != expected_parent {
    return Err(BeaconParentRootMismatch);
}
```

**Why?** Validates chain linkage. For slot 1, uses genesis root. For other slots, must match the previous attestation's beacon_block_root.

### Step 7: Beacon Header Timestamp

```rust
if beacon_header.timestamp != execution_payload.timestamp {
    return Err(BeaconTimestampMismatch);
}
```

**Why?** Cross-validates that header and execution payload are consistent.

### Step 8: Find Current Attestation

```rust
let current_attestation = beacon_block.body.attestations
    .find(|att| att.data.slot == target_slot);
```

**Why?** Need the current slot's attestation for signature verification.

### Step 9: Current Attestation Signature

```rust
verify_attestation_signature(current_attestation, sequencer, Current)?;
```

**Why?** This is the core security check - proves the sequencer signed this beacon_block_root.

### Step 10: Beacon Block Root Validation

```rust
let computed = compute_beacon_block_root(beacon_header);
if computed != current_attestation.data.beacon_block_root {
    return Err(BeaconBlockRootMismatch);
}
```

Beacon block root computation:
```
root = keccak256(rlp([slot, round, timestamp, [proposer],
                       parent_root, state_root, body_root]))
```

**Why?** Links the signed beacon_block_root to the beacon header (which contains body_root).

### Step 11: User Input Validation

```rust
if execution_payload.block_hash != user_block_hash {
    return Err(BlockHashMismatch);
}
```

**Why?** Final step - confirms the user's block hash matches the trusted execution payload.

---

## Attack Surface Analysis

### Can an attacker forge a valid proof?

| Attack Vector | Why It Fails |
|--------------|--------------|
| Fake attestation signature | ECDSA recovery would return wrong address (not sequencer) |
| Modified execution payload | Body hash would not match beacon_header.body_root |
| Modified beacon header | Computed beacon_block_root would not match attestation |
| Different block hash | Would not match execution_payload.block_hash |
| Replay old block | Slot validation ensures correct slot; parent_root ensures chain continuity |
| Malicious RPC | Can only affect availability, not integrity |

### What about extra attestations?

The RPC could include extra attestations beyond the two we need (current and previous). This is harmless because:

1. We only use attestations for specific slots
2. Any unused attestations don't affect the cryptographic chain
3. The verification explicitly finds attestations by slot number

### What about the previous attestation's beacon_block_root?

The previous attestation contains a `beacon_block_root` field that we use for `parent_root` validation. We verify the previous attestation signature (Step 3), so we know it was signed by the sequencer. This creates backward chain linkage.

---

## Verification Modes

### `verify_block_hash` (Minimal Mode)

**Input**: Block hash + RPC responses
**Output**: Success or error
**Proves**: The block hash was committed to by the Linea sequencer

Steps 0-10 validate the beacon chain data, then Step 11 confirms the block hash matches.

### `verify_header` (Full Mode)

**Input**: Complete header + RPC responses
**Output**: Success or error
**Proves**: All header fields match the execution payload

In addition to all checks from minimal mode, this mode:

1. Computes block hash from input header
2. Verifies block hash matches execution payload
3. Verifies all 13 individual fields match:

| Field | Comparison |
|-------|------------|
| `block_number` | `execution_payload.block_number == header.number` |
| `parent_hash` | `execution_payload.parent_hash == header.parent_hash` |
| `state_root` | `execution_payload.state_root == header.state_root` |
| `receipts_root` | `execution_payload.receipts_root == header.receipts_root` |
| `fee_recipient` | `execution_payload.fee_recipient == header.beneficiary` |
| `logs_bloom` | `execution_payload.logs_bloom == header.logs_bloom` |
| `timestamp` | `execution_payload.timestamp == header.timestamp` |
| `gas_limit` | `execution_payload.gas_limit == header.gas_limit` |
| `gas_used` | `execution_payload.gas_used == header.gas_used` |
| `base_fee_per_gas` | `execution_payload.base_fee_per_gas == header.base_fee_per_gas` |
| `extra_data` | `execution_payload.extra_data == header.extra_data` |
| `prev_randao` | `execution_payload.prev_randao == header.mix_hash` |
| `transactions_root` | computed from `execution_payload.transactions` |

**Why full mode?** If the block hash matches but a field differs, it indicates a bug in Linea's implementation. This mode detects such bugs (`LineaBugDetected` error).

---

## Summary

The verifier provides cryptographic proof that a block hash or header was committed to by the Linea sequencer through:

1. **ECDSA signature verification** on attestations
2. **Hash chain validation** from signature → beacon root → body hash → block hash
3. **Chain continuity** via previous attestation validation
4. **Slot linkage** via parent_root validation

The only trust assumptions are the three hardcoded constants (sequencer addresses, activation blocks, genesis beacon roots). Everything else is validated cryptographically.
