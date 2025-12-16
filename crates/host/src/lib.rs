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

//! Host utilities for Linea V4 proofs.
//!
//! This crate contains networking and I/O code that runs on the host
//! (outside of zkVM). It provides beacon chain client and RPC utilities.

mod beacon;
pub mod rpc;

use alloy_eips::BlockNumberOrTag;
use alloy_provider::Provider;
use color_eyre::eyre;
use linea_proof_core::constants::LineaNetwork;
use linea_proof_core::zkvm::VerifyInput;

pub use beacon::Client as BeaconClient;

/// Create a `VerifyInput` from the most recent block on Linea Sepolia.
///
/// This is a convenience function for testing that fetches the latest block
/// from Linea Sepolia and creates a `VerifyInput` ready for zkVM execution.
pub async fn create_verify_input() -> eyre::Result<VerifyInput> {
  create_verify_input_for_network(LineaNetwork::Sepolia).await
}

/// Create a `VerifyInput` from the most recent block on the specified network.
pub async fn create_verify_input_for_network(network: LineaNetwork) -> eyre::Result<VerifyInput> {
  let el_rpc = rpc::get_linea_el_http_rpc(network);
  let cl_rpc = rpc::get_linea_cl_http_rpc(network);

  let rpc_url = el_rpc.try_into()?;
  let provider: alloy_provider::RootProvider<alloy_provider::network::Ethereum> =
    alloy_provider::RootProvider::new_http(rpc_url);

  let latest_block = provider
    .get_block_by_number(BlockNumberOrTag::Latest)
    .await?
    .ok_or_else(|| eyre::eyre!("latest block should exist"))?;
  let header = latest_block.header.inner;

  let beacon_client = BeaconClient::new(cl_rpc, network);
  beacon_client.create_verify_input(header).await
}

/// Load a `VerifyInput` from a file containing serialized bytes.
pub fn load_verify_input_from_file(path: &std::path::Path) -> eyre::Result<VerifyInput> {
  let bytes = std::fs::read(path)?;
  let input = VerifyInput::from_bytes(&bytes)?;
  Ok(input)
}

/// Save a `VerifyInput` to a file as serialized bytes.
pub fn save_verify_input_to_file(input: &VerifyInput, path: &std::path::Path) -> eyre::Result<()> {
  let bytes = input.to_bytes()?;
  std::fs::write(path, bytes)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use linea_proof_core::zkvm::UserInput;

  #[::tokio::test]
  async fn test_verify_input_roundtrip() {
    let input = create_verify_input().await.expect("should fetch input");

    // Serialize to bytes.
    let bytes = input.to_bytes().expect("should serialize");
    println!("Serialized to {} bytes", bytes.len());

    // Deserialize back.
    let restored = VerifyInput::from_bytes(&bytes).expect("should deserialize");

    // Verify fields match.
    let (input_number, restored_number) = match (&input.user_input, &restored.user_input) {
      (UserInput::Header(h1), UserInput::Header(h2)) => (h1.number, h2.number),
      _ => panic!("expected Header variant"),
    };
    assert_eq!(input_number, restored_number);
    assert_eq!(input.network, restored.network);
    assert_eq!(
      input.beacon_data.block_response.data.message.slot,
      restored.beacon_data.block_response.data.message.slot
    );
  }

  mod genesis_beacon_root {
    use super::*;
    use linea_proof_core::compute_beacon_block_root;
    use linea_proof_core::constants::{
      LINEA_MAINNET_GENESIS_BEACON_ROOT, LINEA_SEPOLIA_GENESIS_BEACON_ROOT,
    };

    /// Fetches slot 0 beacon header and computes its beacon_block_root.
    /// Verifies it matches the genesis beacon root constant.
    #[::tokio::test]
    async fn test_mainnet_genesis_beacon_root() {
      let cl_rpc = rpc::get_linea_cl_http_rpc(LineaNetwork::Mainnet);
      let beacon_client = BeaconClient::new(cl_rpc, LineaNetwork::Mainnet);

      // Fetch slot 0 header
      let header_response = beacon_client
        .get_header(0)
        .await
        .expect("should fetch slot 0 header");

      // Compute beacon block root from the header
      let computed_root = compute_beacon_block_root(&header_response.data.message);

      // Verify it matches the constant
      assert_eq!(
        computed_root, LINEA_MAINNET_GENESIS_BEACON_ROOT,
        "Mainnet genesis beacon root mismatch: computed {}, expected {}",
        computed_root, LINEA_MAINNET_GENESIS_BEACON_ROOT
      );

      println!(
        "Mainnet genesis beacon root verified: {}",
        LINEA_MAINNET_GENESIS_BEACON_ROOT
      );
    }

    /// Fetches slot 0 beacon header and computes its beacon_block_root.
    /// Verifies it matches the genesis beacon root constant.
    #[::tokio::test]
    async fn test_sepolia_genesis_beacon_root() {
      let cl_rpc = rpc::get_linea_cl_http_rpc(LineaNetwork::Sepolia);
      let beacon_client = BeaconClient::new(cl_rpc, LineaNetwork::Sepolia);

      // Fetch slot 0 header
      let header_response = beacon_client
        .get_header(0)
        .await
        .expect("should fetch slot 0 header");

      // Compute beacon block root from the header
      let computed_root = compute_beacon_block_root(&header_response.data.message);

      // Verify it matches the constant
      assert_eq!(
        computed_root, LINEA_SEPOLIA_GENESIS_BEACON_ROOT,
        "Sepolia genesis beacon root mismatch: computed {}, expected {}",
        computed_root, LINEA_SEPOLIA_GENESIS_BEACON_ROOT
      );

      println!(
        "Sepolia genesis beacon root verified: {}",
        LINEA_SEPOLIA_GENESIS_BEACON_ROOT
      );
    }
  }
}
