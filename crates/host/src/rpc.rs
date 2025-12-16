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

//! RPC endpoint configuration.

use linea_proof_core::constants::LineaNetwork;

// Our Besu node, running on `c3.large.x86` instance.
pub const LINEA_EL_HTTP_RPC_MAINNET: &str = "http://185.26.8.113:10545";

// Our Besu node, running on `rs4.metal.large` instance.
pub const LINEA_EL_HTTP_RPC_SEPOLIA: &str = "http://103.106.59.59:10545";

// Our Maru node, running on `c3.large.x86` instance.
pub const LINEA_CL_HTTP_RPC_MAINNET: &str = "http://185.26.8.113:10080";

// Our Maru node, running on `rs4.metal.large` instance.
pub const LINEA_CL_HTTP_RPC_SEPOLIA: &str = "http://103.106.59.59:10080";

/// Get the Linea Execution Layer's HTTP RPC URL for a network.
pub fn get_linea_el_http_rpc(network: LineaNetwork) -> &'static str {
  match network {
    LineaNetwork::Mainnet => LINEA_EL_HTTP_RPC_MAINNET,
    LineaNetwork::Sepolia => LINEA_EL_HTTP_RPC_SEPOLIA,
  }
}

/// Get the Linea Consensus Layer's HTTP RPC URL for a network.
/// NOTE: This is often called Beacon API.
pub fn get_linea_cl_http_rpc(network: LineaNetwork) -> &'static str {
  match network {
    LineaNetwork::Mainnet => LINEA_CL_HTTP_RPC_MAINNET,
    LineaNetwork::Sepolia => LINEA_CL_HTTP_RPC_SEPOLIA,
  }
}
