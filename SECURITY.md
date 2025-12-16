# Security Model & Verification Workflow

This document describes the cryptographic verification flow and security model for the Linea Block Verifier.

## Table of Contents

1. [Overview](#overview)
2. [Trust Model](#trust-model)
3. [Verification Workflow](#verification-workflow)
4. [Verification Modes](#verification-modes)
5. [Attack Surface Analysis](#attack-surface-analysis)

---

## Overview

The verifier proves that a Linea block hash (or full header) was signed by the Linea sequencer through the Maru consensus beacon chain. The core insight is:

> **If we can prove that the Linea sequencer signed a beacon_block_root, and we can cryptographically link that root to a specific block hash, then the block hash is authentic.**

The verification builds a chain of cryptographic commitments from the sequencer's ECDSA signature down to the user's block hash:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  CURRENT ATTESTATION SIGNATURE (Step 9)                                     │
│                                                                             │
│  attestation.signature ──► recover_address() ══► must equal SEQUENCER      │
│  attestation.data.beacon_block_root ◄───────────────────────────────────────│
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼ must equal (Step 10)
┌─────────────────────────────────────────────────────────────────────────────┐
│  BEACON BLOCK ROOT                                                          │
│                                                                             │
│  computed = keccak256(rlp([slot, round, timestamp, [proposer],              │
│                            parent_root, state_root, body_root]))            │
│                                    │                                        │
│  beacon_header.body_root ◄─────────│                                        │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼ must equal (Step 4)
┌─────────────────────────────────────────────────────────────────────────────┐
│  BODY HASH                                                                  │
│                                                                             │
│  computed = keccak256(rlp([[prev_attestation.signature],                    │
│                             execution_payload]))                            │
│                                    │                                        │
│  execution_payload.block_hash ◄────│                                        │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼ must equal (Step 11)
┌─────────────────────────────────────────────────────────────────────────────┐
│  USER INPUT                                                                 │
│                                                                             │
│  user's block_hash (or computed from user's header via hash_slow())         │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Key insight**: Each arrow represents a cryptographic commitment. If any link is broken, the verification fails. Only the Linea sequencer can produce a valid chain.

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

Everything else comes from untrusted sources and is validated cryptographically:

| Input | Source | Trust Level |
|-------|--------|-------------|
| `BlockResponse` | Beacon RPC `/eth/v2/beacon/blocks/{slot}` | Untrusted |
| `HeaderResponse` | Beacon RPC `/eth/v1/beacon/headers/{slot}` | Untrusted |
| `block_hash` or `Header` | User input | What we're verifying |
| `network` | Caller specifies | Determines which constants to use |

The RPC can only affect **availability** (refuse to respond), not **integrity** (cannot forge valid data).

---

## Verification Workflow

The verification performs 11 steps (numbered 0-10 in code, plus final user input check):

### Step 0: RPC Sanity Checks

Basic validation of the RPC response format:

```rust
// Version must be "maru"
if block_response.version != "maru" {
    return Err(UnsupportedVersion);
}

// Linea doesn't use BLS signatures, so this should be empty
if !block_response.data.signature.is_empty() {
    return Err(BeaconSignatureNotEmpty);
}

// Proposer must be the expected sequencer
let proposer: Address = beacon_block.proposer_index.parse()?;
if proposer != sequencer {
    return Err(InvalidProposer);
}
```

### Step 1: Slot Validation

```rust
let target_slot = beacon_block.slot;

// Slot 0 is the activation block - no beacon data exists for it
if target_slot == 0 {
    return Err(InvalidSlotZero);
}
```

### Step 2: Find Previous Attestation

For slots > 1, locate the attestation for the previous slot:

```rust
let prev_attestation = if target_slot > 1 {
    Some(
        beacon_block.body.attestations
            .iter()
            .find(|att| att.data.slot == target_slot - 1)
            .ok_or(NoPrevAttestation)?
    )
} else {
    None  // Slot 1 has no previous attestation
};
```

### Step 3: Previous Attestation Signature

Verify the previous attestation was signed by the Linea sequencer (chain continuity):

```rust
if let Some(attestation) = prev_attestation {
    let signature = Signature::try_from(attestation.signature.as_ref())?;
    let recovered = signature.recover_address_from_prehash(
        &attestation.data.beacon_block_root
    )?;

    if recovered != sequencer {
        return Err(PrevAttestationSignerMismatch);
    }
}
```

### Step 4: Body Hash Validation

Compute the body hash and verify it matches `beacon_header.body_root`:

```rust
let computed_body_hash = compute_body_hash(
    prev_attestation,
    &beacon_block.body.execution_payload
);

if computed_body_hash != beacon_header.body_root {
    return Err(BodyHashMismatch);
}
```

Body hash formula:
```
body = rlp([
    [prev_attestation.signature],  // Empty list for slot 1
    execution_payload
])
body_hash = keccak256(body)
```

**At this point**: The `ExecutionPayload` is cryptographically linked to `BeaconHeader.body_root`.

### Step 5: Beacon Header Slot

```rust
if beacon_header.slot != target_slot {
    return Err(BeaconSlotMismatch);
}
```

### Step 6: Beacon Header Parent Root

Verify chain linkage via `parent_root`:

```rust
let expected_parent_root = if target_slot == 1 {
    get_genesis_beacon_root(network)  // Hardcoded constant
} else {
    prev_attestation.unwrap().data.beacon_block_root
};

if beacon_header.parent_root != expected_parent_root {
    return Err(BeaconParentRootMismatch);
}
```

### Step 7: Beacon Header Timestamp

```rust
if beacon_header.timestamp != execution_payload.timestamp {
    return Err(BeaconTimestampMismatch);
}
```

### Step 8: Find Current Attestation

```rust
let current_attestation = beacon_block.body.attestations
    .iter()
    .find(|att| att.data.slot == target_slot)
    .ok_or(NoAttestationForSlot)?;
```

### Step 9: Current Attestation Signature

**This is the core security check.** Verify the current attestation was signed by the Linea sequencer:

```rust
let signature = Signature::try_from(current_attestation.signature.as_ref())?;
let recovered = signature.recover_address_from_prehash(
    &current_attestation.data.beacon_block_root
)?;

if recovered != sequencer {
    return Err(SignerMismatch);
}
```

**At this point**: We know the sequencer signed `current_attestation.data.beacon_block_root`.

### Step 10: Beacon Block Root Validation

Compute the beacon block root from the header and verify it matches what the sequencer signed:

```rust
let computed_beacon_root = compute_beacon_block_root(beacon_header);

if computed_beacon_root != current_attestation.data.beacon_block_root {
    return Err(BeaconBlockRootMismatch);
}
```

Beacon block root formula:
```
root = keccak256(rlp([
    slot, round, timestamp, [proposer],
    parent_root, state_root, body_root
]))
```

**At this point**:
- `BeaconHeader` is fully trusted (its hash was signed by sequencer)
- `ExecutionPayload` is fully trusted (linked via `body_root`)

### Step 11: User Input Validation

Validate the user's input against the trusted execution payload.

```rust
if execution_payload.block_hash != user_block_hash {
    return Err(BlockHashMismatch);
}
```

### Trust Flow Summary

```
Step 0-1:   Sanity checks, determine target slot
Step 2-3:   Find and validate previous attestation (chain continuity)
Step 4:     Link ExecutionPayload to BeaconHeader via body_root
Step 5-7:   Validate BeaconHeader fields (slot, parent_root, timestamp)
Step 8-9:   Find and validate current attestation (CORE SECURITY)
Step 10:    Link BeaconHeader to signed beacon_block_root
Step 11:    Validate user's block hash or header against trusted ExecutionPayload
```

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

| Execution Payload Field | Header Field |
|------------------------|--------------|
| `block_number` | `number` |
| `parent_hash` | `parent_hash` |
| `state_root` | `state_root` |
| `receipts_root` | `receipts_root` |
| `fee_recipient` | `beneficiary` |
| `logs_bloom` | `logs_bloom` |
| `timestamp` | `timestamp` |
| `gas_limit` | `gas_limit` |
| `gas_used` | `gas_used` |
| `base_fee_per_gas` | `base_fee_per_gas` |
| `extra_data` | `extra_data` |
| `prev_randao` | `mix_hash` |
| `transactions` → root | `transactions_root` |

**Why full mode?** If the block hash matches but a field differs, it indicates a bug in Linea's implementation. This mode detects such bugs (`LineaBugDetected` error).

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
