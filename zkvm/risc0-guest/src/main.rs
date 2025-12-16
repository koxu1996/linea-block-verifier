#![no_main]

extern crate alloc;

use linea_block_verifier::core::zkvm::verify_from_bytes;
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

fn main() {
  // Read bincode-serialized input from host.
  let input_bytes = env::read_frame();

  println!("Guest received {} bytes", input_bytes.len());

  // Deserialize and verify (handles both BlockHash and Header variants).
  let block_hash = verify_from_bytes(&input_bytes).expect("verification failed");
  println!("Verification passed! Block hash: {}", block_hash);

  // Commit the verified block hash.
  env::commit(&block_hash);
}
