//! Network constants for Linea.

use alloy_primitives::Address;

/// Linea network identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum LineaNetwork {
  Mainnet,
  Sepolia,
}

pub const LINEA_MAINNET_CHAIN_ID: u64 = 59144;
pub const LINEA_SEPOLIA_CHAIN_ID: u64 = 59141;

pub const LINEA_MAINNET_ACTIVATION_BLOCK: u64 = 24_787_631;
pub const LINEA_SEPOLIA_ACTIVATION_BLOCK: u64 = 18_665_903;

/// Mainnet sequencer address.
pub const LINEA_MAINNET_SEQUENCER: Address = Address::new(hex_literal::hex!(
  "9f31730181441beb67b10efaed5773767ea959bc"
));

/// Sepolia sequencer address.
pub const LINEA_SEPOLIA_SEQUENCER: Address = Address::new(hex_literal::hex!(
  "20e2654209c3f5a7b4728fb228778f1261f1013f"
));

/// Get the chain ID for a network.
pub fn get_chain_id(network: LineaNetwork) -> u64 {
  match network {
    LineaNetwork::Mainnet => LINEA_MAINNET_CHAIN_ID,
    LineaNetwork::Sepolia => LINEA_SEPOLIA_CHAIN_ID,
  }
}

/// Get the activation block for a network.
pub fn get_activation_block(network: LineaNetwork) -> u64 {
  match network {
    LineaNetwork::Mainnet => LINEA_MAINNET_ACTIVATION_BLOCK,
    LineaNetwork::Sepolia => LINEA_SEPOLIA_ACTIVATION_BLOCK,
  }
}

/// Get the expected sequencer address for a network.
pub fn get_sequencer_address(network: LineaNetwork) -> Address {
  match network {
    LineaNetwork::Mainnet => LINEA_MAINNET_SEQUENCER,
    LineaNetwork::Sepolia => LINEA_SEPOLIA_SEQUENCER,
  }
}
