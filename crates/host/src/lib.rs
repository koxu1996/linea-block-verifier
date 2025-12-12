//! Host utilities for Linea V4 proofs.
//!
//! This crate contains networking and I/O code that runs on the host
//! (outside of zkVM). It provides beacon chain client and RPC utilities.

mod beacon;
mod rpc;

use alloy_eips::BlockNumberOrTag;
use alloy_provider::Provider;
use color_eyre::eyre;

pub use beacon::Client as BeaconClient;
pub use rpc::{
  LINEA_CL_HTTP_RPC_MAINNET, LINEA_CL_HTTP_RPC_SEPOLIA, LINEA_EL_HTTP_RPC_MAINNET,
  LINEA_EL_HTTP_RPC_SEPOLIA, get_linea_cl_http_rpc, get_linea_el_http_rpc,
};

// Re-export core types for convenience
pub use linea_proof_core::{
  Attestation, AttestationData, BeaconBlock, BeaconBlockBody, BeaconHeader, BlockData,
  BlockResponse, ExecutionPayload, HeaderData, HeaderResponse, LINEA_MAINNET_CHAIN_ID,
  LINEA_SEPOLIA_CHAIN_ID, LineaNetwork, VerifyError, VerifyInput, beacon_slot_from_block_number,
  get_chain_id, get_sequencer_address, verify_from_bytes, verify_header,
};

/// Create a `VerifyInput` from the most recent block on Linea Sepolia.
///
/// This is a convenience function for testing that fetches the latest block
/// from Linea Sepolia and creates a `VerifyInput` ready for zkVM execution.
pub async fn create_verify_input() -> eyre::Result<VerifyInput> {
  create_verify_input_for_network(LineaNetwork::Sepolia).await
}

/// Create a `VerifyInput` from the most recent block on the specified network.
pub async fn create_verify_input_for_network(network: LineaNetwork) -> eyre::Result<VerifyInput> {
  let el_rpc = get_linea_el_http_rpc(network);
  let cl_rpc = get_linea_cl_http_rpc(network);

  let rpc_url = el_rpc.try_into()?;
  let provider: alloy_provider::RootProvider<alloy_provider::network::Ethereum> =
    alloy_provider::RootProvider::new_http(rpc_url);

  let latest_block = provider
    .get_block_by_number(BlockNumberOrTag::Latest)
    .await?
    .expect("latest block should exist");
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
  let bytes = input.to_bytes();
  std::fs::write(path, bytes)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use crate::{VerifyInput, create_verify_input};

  #[::tokio::test]
  async fn test_verify_input_roundtrip() {
    let input = create_verify_input().await.expect("should fetch input");

    // Serialize to bytes.
    let bytes = input.to_bytes();
    println!("Serialized to {} bytes", bytes.len());

    // Deserialize back.
    let restored = VerifyInput::from_bytes(&bytes).expect("should deserialize");

    // Verify fields match.
    assert_eq!(input.input_header.number, restored.input_header.number);
    assert_eq!(input.network, restored.network);
    assert_eq!(
      input.block_response.data.message.slot,
      restored.block_response.data.message.slot
    );
  }
}
