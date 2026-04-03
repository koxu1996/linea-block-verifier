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
