//! Beacon chain API client.

use alloy_consensus::Header;
use color_eyre::eyre::{self, bail};
use linea_proof_core::{LineaNetwork, VerifyInput, beacon_slot_from_block_number, get_chain_id};
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

  /// Fetch beacon block header response for a given slot.
  pub async fn get_header(&self, slot: u64) -> eyre::Result<linea_proof_core::HeaderResponse> {
    let url = format!("{}/eth/v1/beacon/headers/{}", self.base_url, slot);
    let response = self.http.get(&url).send().await?;

    if !response.status().is_success() {
      let error: ApiError = response.json().await?;
      bail!("Beacon API error ({}): {}", error.code, error.message);
    }

    Ok(response.json().await?)
  }

  /// Fetch full beacon block response for a given slot.
  pub async fn get_block(&self, slot: u64) -> eyre::Result<linea_proof_core::BlockResponse> {
    let url = format!("{}/eth/v2/beacon/blocks/{}", self.base_url, slot);
    let response = self.http.get(&url).send().await?;

    if !response.status().is_success() {
      let error: ApiError = response.json().await?;
      bail!("Beacon API error ({}): {}", error.code, error.message);
    }

    Ok(response.json().await?)
  }

  /// Create a `VerifyInput` for zkVM verification from an execution header.
  ///
  /// This fetches the corresponding beacon block and header for the given
  /// execution header and packages everything into a `VerifyInput` that can
  /// be serialized and passed into a zkVM.
  pub async fn create_verify_input(&self, header: Header) -> eyre::Result<VerifyInput> {
    let chain_id = get_chain_id(self.network);
    let beacon_slot = beacon_slot_from_block_number(header.number, chain_id)?;

    let block_response = self.get_block(beacon_slot).await?;
    let beacon_slot = block_response.data.message.slot;
    let header_response = self.get_header(beacon_slot).await?;

    Ok(VerifyInput::new(
      header,
      block_response,
      header_response,
      self.network,
    ))
  }
}
