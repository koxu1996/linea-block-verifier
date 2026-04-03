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

#![no_main]

extern crate alloc;

use linea_block_verifier::core::constants::get_chain_id;
use linea_block_verifier::core::zkvm::{VerifyInput, verify_from_bytes};
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

fn main() {
  // Read bincode-serialized input from host.
  let input_bytes = env::read_frame();

  println!("Guest received {} bytes", input_bytes.len());

  // Deserialize input to extract network and block number for commitment.
  let input = VerifyInput::from_bytes(&input_bytes).expect("deserialization failed");
  let chain_id = get_chain_id(input.network);
  let block_number = input
    .beacon_data
    .block_response
    .data
    .message
    .body
    .execution_payload
    .block_number;

  // Verify (handles both BlockHash and Header variants).
  let block_hash = verify_from_bytes(&input_bytes).expect("verification failed");
  println!(
    "Verification passed! Block {} ({}) on chain {}",
    block_number, block_hash, chain_id
  );

  // Commit the verified block hash, block number, and chain ID.
  env::commit(&block_hash);
  env::commit(&block_number);
  env::commit(&chain_id);
}
