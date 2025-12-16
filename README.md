# Linea Block Verifier

Cryptographically verify Linea blocks using beacon chain attestations and sequencer signatures.

## Overview

This project provides tools to prove that a Linea block hash or header is authentic by verifying the sequencer's ECDSA signature. It's designed for use in zkVM environments where trustless verification is required.

## Crates

| Crate | Description |
|-------|-------------|
| `linea-proof-core` | Core verification logic (no-std compatible for zkVM) |
| `linea-proof-host` | Host utilities: beacon client, RPC helpers |
| `verify-header-demo` | CLI demo for header verification |

## zkVM

| Crate | Description |
|-------|-------------|
| `zkvm/risc0-guest` | RISC Zero guest program for zkVM verification |
| `zkvm/risc0-host` | RISC Zero host for executing the guest |

## Verification Modes

Two verification modes are available:

| Mode | Description | Use Case |
|------|-------------|----------|
| `verify_block_hash` | Verifies only the block hash | Faster, minimal proof that block hash is authentic |
| `verify_header` | Verifies full header + Linea bug detection | Complete validation with field-by-field checks |

## What This Proves

Given a block hash or header, verification proves:

1. **The Linea sequencer signed a commitment** to that exact block (ECDSA signature)
2. **Current slot attestation** was signed by the sequencer
3. **Previous slot attestation** was signed by the sequencer (chain continuity)
4. **Beacon header is correctly linked** to parent block via parent_root
5. **Body hash commits to execution payload** containing the block hash
6. **No data can be forged** - cryptographic proof chain from block hash → execution payload → body hash → beacon root → attestation signature

## Security Model

### Trust Assumptions

Only three hardcoded constants:
- **Sequencer addresses**: Mainnet and Sepolia sequencer public keys
- **Activation blocks**: Network-specific V4 activation block numbers
- **Genesis beacon roots**: Slot 0 beacon block roots for parent_root validation of slot 1

### Everything Else is Untrusted
- Input block hash/header - what we're verifying
- Beacon RPC responses - validated cryptographically, can only affect availability

### Cryptographic Chain of Trust

```
┌─────────────────────────────────────────────────────────────────┐
│                    CURRENT ATTESTATION                          │
│  signature ─────────► recover_address ═══► must equal sequencer │
│  data.beacon_block_root ◄───────────────────────────────────────│
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ must equal
┌─────────────────────────────────────────────────────────────────┐
│                    BEACON HEADER                                │
│  computed: keccak256(rlp([slot, round, timestamp, [proposer],   │
│            parent_root, state_root, body_root]))                │
│                              │                                   │
│  body_root ◄─────────────────│                                   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ must equal
┌─────────────────────────────────────────────────────────────────┐
│                    BODY HASH                                    │
│  computed: keccak256(rlp([[prev_attestation_sig],               │
│                            execution_payload]))                 │
│                              │                                   │
│  execution_payload.block_hash ◄──────────────────────────────────│
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ must equal
┌─────────────────────────────────────────────────────────────────┐
│                    USER INPUT                                   │
│  block_hash (or computed from header)                           │
└─────────────────────────────────────────────────────────────────┘
```

## Verified Fields (Full Header Mode)

All 12 execution header fields are verified against the beacon execution payload:

| Field | Description |
|-------|-------------|
| `block_number` | Block height |
| `block_hash` | Block identifier |
| `parent_hash` | Link to previous block |
| `state_root` | World state commitment |
| `receipts_root` | Transaction receipts commitment |
| `transactions_root` | Transactions trie root |
| `fee_recipient` | Block proposer/coinbase |
| `logs_bloom` | Event logs bloom filter |
| `timestamp` | Block timestamp |
| `gas_limit` | Maximum gas allowed |
| `gas_used` | Actual gas consumed |
| `base_fee_per_gas` | EIP-1559 base fee |
| `extra_data` | Additional block data |
| `prev_randao` | Previous RANDAO value (mix_hash) |

## Usage

### Demo CLI

```bash
# Verify a recent block on Sepolia
cargo run -p verify-header-demo -- --network sepolia recent

# Verify a recent block on Mainnet
cargo run -p verify-header-demo -- --network mainnet recent

# Verify from local JSON file
cargo run -p verify-header-demo -- --network mainnet local --file-path header.json
```

### Library Usage

```rust
use linea_proof_core::{verify_header, verify_block_hash, LineaNetwork};
use linea_proof_host::BeaconClient;

// Fetch beacon data
let client = BeaconClient::new("http://beacon-rpc:10080", LineaNetwork::Mainnet);
let verify_input = client.create_verify_input(header).await?;

// Option 1: Verify block hash only (faster)
verify_block_hash(block_hash, &beacon_data, LineaNetwork::Mainnet)?;

// Option 2: Verify full header with Linea bug detection
verify_header(&input_header, &beacon_data, LineaNetwork::Mainnet)?;
```

### RISC Zero zkVM

The `zkvm/` directory contains RISC Zero integration for generating zero-knowledge proofs of header verification.

```bash
# Build and run the RISC Zero host
cd zkvm/risc0-host
cargo run --release
```

The host will:
1. Fetch a recent Linea block and beacon data (or load from `verify_input.bin` if cached)
2. Execute **both** verification modes in the zkVM
3. Report cycle counts and timing comparison

Example output:
```
Validating Linea block number: 22098808
Block hash: 0x8616e9df03beebc32579b6a622ace9b9c103906deeace76567edf2458d97070b

=== Mode 1: Block Hash Verification ===
BlockHash input: 2465 bytes
Executing...
BlockHash verification: 0.15s, 1355408 cycles, 1 segments

=== Mode 2: Full Header Verification ===
FullHeader input: 3291 bytes
Executing...
FullHeader verification: 0.15s, 1398539 cycles, 1 segments

=== Summary for block 22098808 ===
Block hash: 0x8616e9df03beebc32579b6a622ace9b9c103906deeace76567edf2458d97070b

  BlockHash mode:        1355408 cycles  (0.15s)
  FullHeader mode:       1398539 cycles  (0.15s)

  Difference:             +43131 cycles  (+3.2%)

Both verification modes passed successfully!
```

## Verification Steps

The verification performs these steps:

0. **RPC Sanity Checks**: Version is "maru", block signature is empty, proposer matches sequencer
1. **Slot Validation**: Reject slot 0 (activation block)
2. **Previous Attestation**: Find attestation for slot-1 (required for slots > 1)
3. **Previous Attestation Signature**: Verify signed by sequencer
4. **Body Hash**: Compute and verify against beacon header body_root
5. **Beacon Header Slot**: Verify matches block response slot
6. **Beacon Header Parent Root**: Verify matches previous attestation (or genesis root for slot 1)
7. **Beacon Header Timestamp**: Verify matches execution payload timestamp
8. **Current Attestation**: Find attestation for current slot
9. **Current Attestation Signature**: Verify signed by sequencer
10. **Beacon Block Root**: Compute from header and verify matches attestation
11. **User Input**: Verify block hash (both modes) + field comparison (full header mode)

## Building

```bash
cargo build --release
```

## Nix

A `flake.nix` is provided for reproducible builds, `no_std` compatibility checking, linting, and formatting.

```bash
# Build verify-header-demo
nix build .#verify-header-demo

# Build linea-proof-core for thumbv7em-none-eabi (verifies no_std compatibility)
nix build .#linea-proof-core-no-std

# Run all checks (builds + clippy + formatting)
nix flake check

# Format code (Nix + Rust)
nix fmt

# Enter development shell
nix develop
```

## Testing

```bash
cargo test
```

## References

- [Linea V4 Beta Migration Guide](https://docs.linea.build/get-started/how-to/run-a-node/beta-v4-migration)
- [Maru Consensus](https://github.com/Consensys/maru/)
- [Linea Documentation](https://docs.linea.build/)

## License

MIT OR Apache-2.0
