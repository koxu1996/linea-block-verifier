#![no_main]

extern crate alloc;

use linea_proof_core::VerifyMode;
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

fn main() {
  // Read verification mode from host.
  let mode: VerifyMode = env::read();

  // Read bincode-serialized input from host.
  let input_bytes = env::read_frame();

  println!(
    "Guest received {} bytes, mode: {:?}",
    input_bytes.len(),
    mode
  );

  // Deserialize and verify based on mode.
  match mode {
    VerifyMode::BlockHash => {
      linea_proof_core::verify_block_hash_from_bytes(&input_bytes)
        .expect("block hash verification failed");
      println!("Block hash verification passed!");
    }
    VerifyMode::FullHeader => {
      linea_proof_core::verify_from_bytes(&input_bytes).expect("header verification failed");
      println!("Full header verification passed!");
    }
  }
}
