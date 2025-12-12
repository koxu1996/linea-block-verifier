# Linea Header Verifier

Cryptographically verify Linea block headers using beacon chain attestations and sequencer signatures.

## Overview

This project provides tools to prove that a Linea block header is authentic by verifying the sequencer's ECDSA signature. It's designed for use in zkVM environments where trustless verification is required.

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

## What This Proves

Given a block header, verification proves:

1. **The Linea sequencer signed a commitment** to that exact header (ECDSA signature)
2. **All 12 execution header fields** were committed to by the sequencer
3. **No data can be forged** - cryptographic proof chain from header → body hash → beacon root → signature

## Security Model

### Trust Assumptions

Only two hardcoded constants:
- **Sequencer addresses**: Mainnet and Sepolia sequencer public keys
- **Activation blocks**: Network-specific V4 activation block numbers

### Everything Else is Untrusted
- Input block header - what we're verifying
- Beacon RPC endpoints - can only affect availability, not correctness

## Verified Fields

All 12 execution header fields are verified:

| Field | Description |
|-------|-------------|
| `block_number` | Block height |
| `block_hash` | Block identifier |
| `parent_hash` | Link to previous block |
| `state_root` | World state commitment |
| `receipts_root` | Transaction receipts commitment |
| `fee_recipient` | Block proposer/coinbase |
| `logs_bloom` | Event logs bloom filter |
| `timestamp` | Block timestamp |
| `gas_limit` | Maximum gas allowed |
| `gas_used` | Actual gas consumed |
| `base_fee_per_gas` | EIP-1559 base fee |
| `extra_data` | Additional block data |

## Usage

### Demo CLI

```bash
# Verify a recent block on Sepolia (5 blocks behind head)
cargo run --bin verify-header-demo -- --network sepolia recent

# Verify a recent block on Mainnet
cargo run --bin verify-header-demo -- --network mainnet recent

# Verify from local JSON file
cargo run --bin verify-header-demo -- --network sepolia local --file-path header.json
```

### Library Usage

```rust
use linea_proof_core::{verify_header, LineaNetwork};
use linea_proof_host::BeaconClient;

// Fetch beacon data
let client = BeaconClient::new("http://beacon-rpc:10080", LineaNetwork::Mainnet);
let beacon_block = client.get_block(slot).await?;
let beacon_header = client.get_header(slot).await?;

// Verify header
verify_header(&input_header, &beacon_block, &beacon_header, LineaNetwork::Mainnet)?;
```

### RISC Zero zkVM

The `zkvm/` directory contains RISC Zero integration for generating zero-knowledge proofs of header verification.

```bash
# Build and run the RISC Zero host (fetches input on first run, caches to verify_input.bin)
cd zkvm/risc0-host
cargo run --release

# Subsequent runs use cached input for reproducibility
cargo run --release
```

The host will:
1. Fetch a recent Linea block and beacon data (or load from `verify_input.bin` if present)
2. Execute the guest program in the zkVM
3. Report cycle counts and segment information

Example output:
```
Loading cached input from verify_input.bin...
WARNING: Using cached input. Remove verify_input.bin to fetch new data for recent blocks.
Validating Linea block number: 21919045
Input serialized to 3275 bytes
Generating traces...
Guest received 3275 bytes
Verification passed!
Traces ready - it took 0.13 seconds
- num segments: 1
- total cycles: 780048

Linea block 21919045 verified successfully in zkVM!
```

## How It Works

```
Input Block Header
        ↓
Fetch Beacon Block (for matching block number)
        ↓
Compare all 12 fields against execution_payload
        ↓
Compute body_root = keccak256(rlp([prevCommitSeals, executionPayload]))
        ↓
Compute beacon_block_root = keccak256(rlp(header))
        ↓
Verify attestation references correct beacon_block_root
        ↓
Recover signer from ECDSA signature
        ↓
Verify signer == expected sequencer
        ↓
✓ Header is authentic
```

## Building

```bash
cargo build --release
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
