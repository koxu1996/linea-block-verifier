use alloy_provider::Provider;
use clap::Parser;
use color_eyre::eyre;

use linea_block_verifier::core::constants::{LineaNetwork, get_chain_id};
use linea_block_verifier::core::{verify_block_hash, verify_header};
use linea_block_verifier::host::BeaconClient;
use linea_block_verifier::host::rpc::{get_linea_cl_http_rpc, get_linea_el_http_rpc};

mod cli;
use cli::{Args, Command, commands};

#[tokio::main]
async fn main() -> eyre::Result<()> {
  let args = Args::parse();

  let linea_network: LineaNetwork = args.network.clone().into();
  let expected_chain_id = get_chain_id(linea_network);

  // Create RPC provider
  let linea_el_rpc = get_linea_el_http_rpc(linea_network);
  let linea_cl_rpc = get_linea_cl_http_rpc(linea_network);

  let rpc_url = linea_el_rpc.try_into()?;
  let provider: alloy_provider::RootProvider<alloy_provider::network::Ethereum> =
    alloy_provider::RootProvider::new_http(rpc_url);
  let chain_id = provider.get_chain_id().await?;
  assert_eq!(chain_id, expected_chain_id);

  // Create beacon client
  let beacon_client = BeaconClient::new(linea_cl_rpc, linea_network);

  match &args.command {
    Command::Recent => {
      let input_header_rpc = commands::recent(&provider).await?;
      verify_header_command(&beacon_client, input_header_rpc.into_consensus()).await?;
    }
    Command::Local { file_path } => {
      let input_header_rpc = commands::local(file_path)?;
      verify_header_command(&beacon_client, input_header_rpc.into_consensus()).await?;
    }
    Command::Hash { block_hash } => {
      verify_hash_command(&provider, &beacon_client, *block_hash).await?;
    }
  }

  Ok(())
}

async fn verify_header_command(
  beacon_client: &BeaconClient,
  input_header: alloy_consensus::Header,
) -> eyre::Result<()> {
  println!("Block number: {}", input_header.number);
  println!("Block hash: {}", input_header.hash_slow());
  println!();

  // Fetch beacon data
  println!("Fetching beacon data...");
  let beacon_data = beacon_client.fetch_beacon_data(input_header.number).await?;
  println!(
    "Beacon block slot: {}",
    beacon_data.block_response.data.message.slot
  );

  // Verify the header against beacon chain data
  verify_header(
    &input_header,
    &beacon_data.block_response,
    &beacon_data.header_response,
    beacon_client.network(),
  )?;

  println!("\n=== All verifications passed! ===");
  println!("1. All 12 execution header fields verified against beacon chain");
  println!("2. Beacon block signature is empty (Linea-specific)");
  println!("3. Proposer address matches expected sequencer");
  println!("4. Beacon block root correctly computed and matches attestation");
  println!("5. Body hash verified: execution_payload cryptographically committed");
  println!("6. Attestation ECDSA signature verified by sequencer");
  println!("\n==> PROVEN: The input header is AUTHENTIC");
  println!("    The Linea sequencer cryptographically signed this exact block header.");
  println!("    All fields (state_root, receipts_root, etc.) can now be trusted.");

  Ok(())
}

async fn verify_hash_command(
  provider: &alloy_provider::RootProvider<alloy_provider::network::Ethereum>,
  beacon_client: &BeaconClient,
  block_hash: alloy_primitives::B256,
) -> eyre::Result<()> {
  // Look up block number from hash
  println!("Looking up block number for hash {}...", block_hash);
  let block = provider
    .get_block_by_hash(block_hash)
    .await?
    .ok_or_else(|| eyre::eyre!("Block not found for hash {}", block_hash))?;
  let block_number = block.header.number;

  println!("Block number: {}", block_number);
  println!("Block hash: {}", block_hash);
  println!();

  // Fetch beacon data
  println!("Fetching beacon data...");
  let beacon_data = beacon_client.fetch_beacon_data(block_number).await?;
  println!(
    "Beacon block slot: {}",
    beacon_data.block_response.data.message.slot
  );

  // Verify only the block hash against beacon chain data
  verify_block_hash(
    block_hash,
    &beacon_data.block_response,
    &beacon_data.header_response,
    beacon_client.network(),
  )?;

  println!("\n=== Block hash verification passed! ===");
  println!("1. Block hash verified against beacon chain execution_payload");
  println!("2. Beacon block signature is empty (Linea-specific)");
  println!("3. Proposer address matches expected sequencer");
  println!("4. Beacon block root correctly computed and matches attestation");
  println!("5. Body hash verified: execution_payload cryptographically committed");
  println!("6. Attestation ECDSA signature verified by sequencer");
  println!("\n==> PROVEN: The block hash {} is AUTHENTIC", block_hash);
  println!("    The Linea sequencer cryptographically signed a block with this hash.");

  Ok(())
}
