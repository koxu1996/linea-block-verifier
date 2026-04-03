use alloy_rpc_types::Header;
use color_eyre::eyre;
use std::path::Path;

pub fn run(file_path: &Path) -> eyre::Result<Header> {
  let header_json = std::fs::read_to_string(file_path)?;
  serde_json::from_str(&header_json)
    .map_err(|e| eyre::eyre!("Failed to parse block header JSON: {}", e))
}
