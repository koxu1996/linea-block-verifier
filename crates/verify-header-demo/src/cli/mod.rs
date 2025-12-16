use clap::{Parser, Subcommand, ValueEnum};
use linea_block_verifier::core::constants::LineaNetwork;
use std::path::PathBuf;

pub mod commands;

/// Verify Linea block header authenticity via sequencer signature.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
  /// Network to use
  #[arg(long, short, default_value = "sepolia")]
  pub network: Network,

  #[command(subcommand)]
  pub command: Command,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Network {
  Mainnet,
  Sepolia,
}

#[derive(Debug, Subcommand)]
pub enum Command {
  /// Fetch and verify a recent block from RPC (5 blocks behind head)
  Recent,
  /// Verify a block header from a local JSON file
  Local {
    /// Path to block header JSON file
    #[arg(long)]
    file_path: PathBuf,
  },
}

impl From<Network> for LineaNetwork {
  fn from(network: Network) -> Self {
    match network {
      Network::Mainnet => LineaNetwork::Mainnet,
      Network::Sepolia => LineaNetwork::Sepolia,
    }
  }
}
