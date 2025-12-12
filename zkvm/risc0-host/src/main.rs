pub const RISC0_GUEST_ELF: &[u8] = include_bytes!(env!(concat!("R0_ELF_", "risc0-guest")));

use risc0_zkvm::{ExecutorEnv, default_executor};
use std::path::Path;

// 21 is the max value for 20GB card like RTX 4090.
const MAX_PO2: u32 = 21;

// Path for cached verify input.
const INPUT_FILE: &str = "verify_input.bin";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
    .init();

  // Load from file if it exists, otherwise fetch from network and save.
  let input_path = Path::new(INPUT_FILE);
  let input = if input_path.exists() {
    println!("Loading cached input from {}...", INPUT_FILE);
    println!(
      "WARNING: Using cached input. Remove {} to fetch new data for recent blocks.",
      INPUT_FILE
    );
    linea_proof_host::load_verify_input_from_file(input_path).map_err(|e| anyhow::anyhow!("{e}"))?
  } else {
    println!("Fetching input from network...");
    let input = linea_proof_host::create_verify_input()
      .await
      .map_err(|e| anyhow::anyhow!("{e}"))?;
    linea_proof_host::save_verify_input_to_file(&input, input_path)
      .map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("Saved input to {}", INPUT_FILE);
    input
  };

  println!(
    "Validating Linea block number: {}",
    input.input_header.number
  );

  // Serialize with bincode.
  let input_bytes = input.to_bytes();
  println!("Input serialized to {} bytes", input_bytes.len());

  // Execute the guest code.
  let env = ExecutorEnv::builder()
    .segment_limit_po2(MAX_PO2)
    .write_frame(&input_bytes)
    .build()?;
  let exec = default_executor();
  println!("Generating traces...");
  let start = std::time::Instant::now();
  let session_info = exec.execute(env, RISC0_GUEST_ELF)?;
  let duration = start.elapsed();
  println!(
    "Traces ready - it took {:.2} seconds",
    duration.as_secs_f64()
  );
  println!("- num segments: {}", session_info.segments.len());
  println!("- total cycles: {}", session_info.cycles());

  println!(
    "\nLinea block {} verified successfully in zkVM!",
    input.input_header.number
  );

  Ok(())
}
