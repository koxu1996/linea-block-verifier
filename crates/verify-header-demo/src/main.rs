use alloy_provider::Provider;
use clap::Parser;
use color_eyre::eyre;

use linea_block_verifier::core::constants::{LineaNetwork, get_chain_id};
use linea_block_verifier::core::verify_header;
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

  let input_header_rpc = match &args.command {
    Command::Recent => commands::recent(&provider).await?,
    Command::Local { file_path } => commands::local(file_path)?,
  };
  let input_header = input_header_rpc.into_consensus();

  println!("Block number: {}", input_header.number);
  println!("Block hash: {}", input_header.hash_slow());
  println!();

  // Create verify input (fetches beacon block and header)
  println!("Fetching beacon data...");
  let verify_input = beacon_client.create_verify_input(input_header).await?;
  println!(
    "Beacon block slot: {}",
    verify_input.block_response.data.message.slot
  );

  // Verify the header against beacon chain data
  verify_header(
    &verify_input.input_header,
    &verify_input.block_response,
    &verify_input.header_response,
    verify_input.network,
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
