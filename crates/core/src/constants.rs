//! Network constants for Linea.

use alloy_primitives::{Address, B256};

/// Linea network identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum LineaNetwork {
  Mainnet,
  Sepolia,
}

impl TryFrom<u64> for LineaNetwork {
  type Error = &'static str;

  fn try_from(chain_id: u64) -> Result<Self, Self::Error> {
    match chain_id {
      LINEA_MAINNET_CHAIN_ID => Ok(LineaNetwork::Mainnet),
      LINEA_SEPOLIA_CHAIN_ID => Ok(LineaNetwork::Sepolia),
      _ => Err("unknown Linea network"),
    }
  }
}

pub const LINEA_MAINNET_CHAIN_ID: u64 = 59144;
pub const LINEA_SEPOLIA_CHAIN_ID: u64 = 59141;

/// Block where consensus was enabled in Linea.
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

/// Mainnet genesis beacon block root (slot 0).
/// This is the beacon_block_root for the activation block, used as parent_root for slot 1.
pub const LINEA_MAINNET_GENESIS_BEACON_ROOT: B256 = B256::new(hex_literal::hex!(
  "c05c72649c12578368f1ab42ebfcc4ea0f0d5b9501d64ef84078f8f40f825892"
));

/// Sepolia genesis beacon block root (slot 0).
/// This is the beacon_block_root for the activation block, used as parent_root for slot 1.
pub const LINEA_SEPOLIA_GENESIS_BEACON_ROOT: B256 = B256::new(hex_literal::hex!(
  "551a114ec84d773a0834ff053a2b964e21cf99a578c9e26159cf791271ee88a2"
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

/// Get the genesis beacon block root (slot 0) for a network.
/// This is used as parent_root for slot 1.
pub fn get_genesis_beacon_root(network: LineaNetwork) -> B256 {
  match network {
    LineaNetwork::Mainnet => LINEA_MAINNET_GENESIS_BEACON_ROOT,
    LineaNetwork::Sepolia => LINEA_SEPOLIA_GENESIS_BEACON_ROOT,
  }
}
