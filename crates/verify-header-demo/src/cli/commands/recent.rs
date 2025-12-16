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
use alloy_rpc_types::Header;
use color_eyre::eyre;

pub async fn run(
  provider: &alloy_provider::RootProvider<alloy_provider::network::Ethereum>,
) -> eyre::Result<Header> {
  // Fetch a recent block (with some buffer to ensure beacon data is available)
  let latest_block_num = provider.get_block_number().await?;
  let block_num = latest_block_num.saturating_sub(5);

  let block = provider
    .get_block_by_number(block_num.into())
    .await?
    .ok_or_else(|| eyre::eyre!("Block {} not found", block_num))?;

  Ok(block.header)
}
