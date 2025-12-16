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

use alloy_rpc_types::Header;
use color_eyre::eyre;
use std::path::Path;

pub fn run(file_path: &Path) -> eyre::Result<Header> {
  let header_json = std::fs::read_to_string(file_path)?;
  serde_json::from_str(&header_json)
    .map_err(|e| eyre::eyre!("Failed to parse block header JSON: {}", e))
}
