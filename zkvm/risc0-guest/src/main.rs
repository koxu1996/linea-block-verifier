#![no_main]

extern crate alloc;

use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

fn main() {
  // Read bincode-serialized VerifyInput from host.
  let input_bytes = env::read_frame();

  // Debug: print received length.
  println!("Guest received {} bytes", input_bytes.len());

  // Deserialize and verify.
  linea_proof_core::verify_from_bytes(&input_bytes).expect("verification failed");

  println!("Verification passed!");
}
