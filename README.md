# Linea Block Verifier

Cryptographically verify Linea blocks using beacon chain attestations and sequencer signatures.

## Overview

This project provides tools to prove that a Linea block hash or header is authentic by verifying the sequencer's ECDSA signature. It's designed for use in zkVM environments where trustless verification is required.

For detailed security model and verification logic, see [SECURITY.md](SECURITY.md).

## Crates

| Crate | Description |
|-------|-------------|
| `linea-block-verifier` | Umbrella crate re-exporting core and host |
| `linea-proof-core` | Core verification logic (for zkVM) and basic types |
| `linea-proof-host` | Host utilities: beacon client, RPC helpers |
| `verify-header-demo` | CLI demo for header verification |

## zkVM

| Crate | Description |
|-------|-------------|
| `zkvm/risc0-guest` | RISC Zero guest program for zkVM verification |
| `zkvm/risc0-host` | RISC Zero host for executing the guest |

## Verification Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| `verify_block_hash` | Verifies only the block hash | Faster, minimal proof that block hash is authentic |
| `verify_header` | Verifies full header + Linea bug detection | Complete validation with field-by-field checks |

## Usage

### Demo CLI

```bash
# Verify a recent block on Sepolia
cargo run -p verify-header-demo -- --network sepolia recent

# Verify a recent block on Mainnet
cargo run -p verify-header-demo -- --network mainnet recent

# Verify a block hash only (without full header)
cargo run -p verify-header-demo -- --network mainnet hash \
  --block-hash 0x6d93533ba354440c035355657195dd7832df0ac3d232904d08e67296fd5ce75c

# Verify from local JSON file
cargo run -p verify-header-demo -- --network mainnet local --file-path header.json
```

### Library Usage

Use the umbrella crate for convenience:

```toml
# For host applications (includes BeaconClient)
linea-block-verifier = { version = "0.1", features = ["host"] }

# For zkVM guests (no_std compatible)
linea-block-verifier = { version = "0.1", features = ["std"] }
```

```rust
use linea_block_verifier::core::{verify_header, verify_block_hash, LineaNetwork};
use linea_block_verifier::host::BeaconClient;

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

This project is licensed under the Business Source License 1.1. See [LICENSE-BSL.md](LICENSE-BSL.md) for details.
