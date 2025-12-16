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

//! Beacon chain API client.

use alloy_consensus::Header;
use alloy_primitives::B256;
use color_eyre::eyre::{self, bail};
use linea_proof_core::beacon_slot_from_block_number;
use linea_proof_core::constants::{LineaNetwork, get_chain_id};
use linea_proof_core::types::{BeaconBlockResponse, BeaconData, BeaconHeaderResponse};
use linea_proof_core::zkvm::VerifyInput;
use serde::Deserialize;

/// Error response from the beacon API.
#[derive(Debug, Deserialize)]
struct ApiError {
  code: u16,
  message: String,
}

/// Client for interacting with Linea's beacon chain API.
pub struct Client {
  base_url: String,
  http: reqwest::Client,
  network: LineaNetwork,
}

impl Client {
  pub fn new(base_url: impl Into<String>, network: LineaNetwork) -> Self {
    Self {
      base_url: base_url.into(),
      http: reqwest::Client::new(),
      network,
    }
  }

  /// Get the network this client is configured for.
  pub fn network(&self) -> LineaNetwork {
    self.network
  }

  /// Fetch beacon block header response for a given slot.
  pub async fn get_header(&self, slot: u64) -> eyre::Result<BeaconHeaderResponse> {
    let url = format!("{}/eth/v1/beacon/headers/{}", self.base_url, slot);
    let response = self.http.get(&url).send().await?;

    if !response.status().is_success() {
      let error: ApiError = response.json().await?;
      bail!("Beacon API error ({}): {}", error.code, error.message);
    }

    Ok(response.json().await?)
  }

  /// Fetch full beacon block response for a given slot.
  pub async fn get_block(&self, slot: u64) -> eyre::Result<BeaconBlockResponse> {
    let url = format!("{}/eth/v2/beacon/blocks/{}", self.base_url, slot);
    let response = self.http.get(&url).send().await?;

    if !response.status().is_success() {
      let error: ApiError = response.json().await?;
      bail!("Beacon API error ({}): {}", error.code, error.message);
    }

    Ok(response.json().await?)
  }

  /// Fetch beacon data for a given block number.
  ///
  /// This fetches the beacon block and header responses needed for verification.
  pub async fn fetch_beacon_data(&self, block_number: u64) -> eyre::Result<BeaconData> {
    let chain_id = get_chain_id(self.network);
    let beacon_slot = beacon_slot_from_block_number(block_number, chain_id)?;

    let block_response = self.get_block(beacon_slot).await?;
    let beacon_slot = block_response.data.message.slot;
    let header_response = self.get_header(beacon_slot).await?;

    Ok(BeaconData {
      block_response,
      header_response,
    })
  }

  /// Create a `VerifyInput` for zkVM verification from an execution header.
  ///
  /// This fetches the corresponding beacon data and packages everything into
  /// a `VerifyInput` that can be serialized and passed into a zkVM.
  pub async fn create_verify_input(&self, header: Header) -> eyre::Result<VerifyInput> {
    let beacon_data = self.fetch_beacon_data(header.number).await?;
    Ok(VerifyInput::new_header(header, beacon_data, self.network))
  }

  /// Create a `VerifyInput` for zkVM verification from a block hash.
  ///
  /// This fetches the corresponding beacon data and packages everything into
  /// a `VerifyInput` that can be serialized and passed into a zkVM.
  pub async fn create_verify_input_for_hash(
    &self,
    block_hash: B256,
    block_number: u64,
  ) -> eyre::Result<VerifyInput> {
    let beacon_data = self.fetch_beacon_data(block_number).await?;
    Ok(VerifyInput::new_block_hash(
      block_hash,
      beacon_data,
      self.network,
    ))
  }
}
