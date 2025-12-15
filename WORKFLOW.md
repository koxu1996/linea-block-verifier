# Verification Workflow

Step-by-step walkthrough of the verification process.

## Inputs

| Input | Source | Trust Level |
|-------|--------|-------------|
| `BlockResponse` | Beacon RPC `/eth/v2/beacon/blocks/{slot}` | Untrusted |
| `HeaderResponse` | Beacon RPC `/eth/v1/beacon/headers/{slot}` | Untrusted |
| `block_hash` or `Header` | User input | What we're verifying |
| `network` | Caller specifies | Determines which constants to use |

## Step 0: RPC Sanity Checks

Basic validation of the RPC response format. These are sanity checks, not security checks - they just ensure we're talking to a properly configured Maru node.

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

## Step 1: Determine Target Slot

Take the slot from `BlockResponse` as our reference. We use this instead of deriving from block number because we want to support block hash verification (where we don't have the full header).

```rust
let target_slot = beacon_block.slot;

// Slot 0 is the activation block - no beacon data exists for it
if target_slot == 0 {
    return Err(InvalidSlotZero);
}
```

## Step 2: Find Previous Attestation

For slots > 1, locate the attestation for the previous slot. This attestation's signature is included in the body hash computation.

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

## Step 3: Validate Previous Attestation Signature

Verify the previous attestation was signed by the Linea sequencer. This ensures chain continuity.

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

## Step 4: Compute Body Hash

Compute the body hash using the previous attestation (or None for slot 1) and the execution payload.

```rust
let computed_body_hash = compute_body_hash(
    prev_attestation,
    &beacon_block.body.execution_payload
);
```

Body hash formula:
```
body = rlp([
    [prev_attestation.signature],  // Empty list for slot 1
    execution_payload
])
body_hash = keccak256(body)
```

## Step 5: Validate Body Hash

Cross-check computed body hash against the `body_root` in the beacon header.

```rust
if computed_body_hash != beacon_header.body_root {
    return Err(BodyHashMismatch);
}
```

**At this point**: If body hash matches, the `ExecutionPayload` is cryptographically linked to `BeaconHeader.body_root`.

## Step 6: Validate Header Slot

Ensure the beacon header corresponds to our target slot.

```rust
if beacon_header.slot != target_slot {
    return Err(BeaconSlotMismatch);
}
```

## Step 7: Validate Header Parent Root

Verify chain linkage via `parent_root`:

- **Slot 1**: Use hardcoded genesis beacon root (computed from slot 0 header)
- **Slots > 1**: Must match previous attestation's `beacon_block_root`

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

## Step 7.5: Validate Header Timestamp

Cross-validate that beacon header timestamp matches execution payload timestamp.

```rust
if beacon_header.timestamp != execution_payload.timestamp {
    return Err(BeaconTimestampMismatch);
}
```

## Step 8: Find Current Attestation

Locate the attestation for the current slot. This always exists.

```rust
let current_attestation = beacon_block.body.attestations
    .iter()
    .find(|att| att.data.slot == target_slot)
    .ok_or(NoAttestationForSlot)?;
```

## Step 9: Validate Current Attestation Signature

**This is the core security check.** Verify the current attestation was signed by the Linea sequencer.

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

## Step 10: Validate Beacon Block Root

Compute the beacon block root from the header and verify it matches what the sequencer signed.

```rust
let computed_beacon_root = compute_beacon_block_root(beacon_header);

if computed_beacon_root != current_attestation.data.beacon_block_root {
    return Err(BeaconBlockRootMismatch);
}
```

Beacon block root formula:
```
root = keccak256(rlp([
    slot,
    round,
    timestamp,
    [proposer],
    parent_root,
    state_root,
    body_root
]))
```

**At this point**:
- `BeaconHeader` is fully trusted (its hash was signed by sequencer)
- `ExecutionPayload` is fully trusted (linked via `body_root`)

## Step 11: Validate User Input

Now we validate what the user actually wants to verify.

### Mode A: Block Hash Only (`verify_block_hash`)

Simply compare the user's block hash against the trusted execution payload:

```rust
if execution_payload.block_hash != user_block_hash {
    return Err(BlockHashMismatch);
}
// Success! Block hash is authentic.
```

### Mode B: Full Header (`verify_header`)

First compute and verify block hash, then check all fields:

```rust
// Compute hash from user's header
let input_block_hash = input_header.hash_slow();

// Verify hash matches
if execution_payload.block_hash != input_block_hash {
    return Err(BlockHashMismatch);
}

// Block hash matches - now verify all fields
// If hash matches but fields differ, it's a Linea bug
verify_execution_payload_fields(execution_payload, input_header)?;
```

Field checks (returns `LineaBugDetected` on mismatch):

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

---

## Trust Flow Summary

```
Step 0-1:   Sanity checks, determine target slot
Step 2-3:   Find and validate previous attestation (chain continuity)
Step 4-5:   Link ExecutionPayload to BeaconHeader via body_root
Step 6-7.5: Validate BeaconHeader fields (slot, parent_root, timestamp)
Step 8-9:   Find and validate current attestation (CORE SECURITY)
Step 10:    Link BeaconHeader to signed beacon_block_root
Step 11:    Validate user's block hash or header against trusted ExecutionPayload
```

After Step 10, we have:
- **Trusted `BeaconHeader`** - its hash was signed by the sequencer
- **Trusted `ExecutionPayload`** - cryptographically linked via `body_root`
- **Trusted `block_hash`** - contained in the trusted execution payload

Step 11 simply confirms the user's input matches what we've proven to be authentic.
