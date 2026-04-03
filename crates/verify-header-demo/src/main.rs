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
  let linea_el_rpc = args
    .el_rpc
    .as_deref()
    .unwrap_or_else(|| get_linea_el_http_rpc(linea_network));
  let linea_cl_rpc = args
    .cl_rpc
    .as_deref()
    .unwrap_or_else(|| get_linea_cl_http_rpc(linea_network));

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
  let block_number = input_header.number;
  let block_hash = input_header.hash_slow();

  let beacon_data = beacon_client.fetch_beacon_data(block_number).await?;
  verify_header(&input_header, &beacon_data, beacon_client.network())?;

  println!("Block {} ({}) is valid", block_number, block_hash);
  Ok(())
}

async fn verify_hash_command(
  provider: &alloy_provider::RootProvider<alloy_provider::network::Ethereum>,
  beacon_client: &BeaconClient,
  block_hash: alloy_primitives::B256,
) -> eyre::Result<()> {
  let block = provider
    .get_block_by_hash(block_hash)
    .await?
    .ok_or_else(|| eyre::eyre!("Block not found for hash {}", block_hash))?;
  let block_number = block.header.number;

  let beacon_data = beacon_client.fetch_beacon_data(block_number).await?;
  verify_block_hash(block_hash, &beacon_data, beacon_client.network())?;

  println!("Block {} ({}) is valid", block_number, block_hash);
  Ok(())
}
