pub const RISC0_GUEST_ELF: &[u8] = include_bytes!(env!(concat!("R0_ELF_", "risc0-guest")));

use linea_proof_host::VerifyMode;
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

  let block_number = input.input_header.number;
  let block_hash = input.input_header.hash_slow();

  println!("Validating Linea block number: {}", block_number);
  println!("Block hash: {}", block_hash);
  println!();

  // =========================================================================
  // Run BlockHash verification mode
  // =========================================================================
  println!("=== Mode 1: Block Hash Verification ===");

  let block_hash_input = input.to_block_hash_input();
  let block_hash_bytes = block_hash_input.to_bytes()?;
  println!("BlockHash input: {} bytes", block_hash_bytes.len());

  let env = ExecutorEnv::builder()
    .segment_limit_po2(MAX_PO2)
    .write(&VerifyMode::BlockHash)?
    .write_frame(&block_hash_bytes)
    .build()?;

  let exec = default_executor();
  println!("Executing...");
  let start = std::time::Instant::now();
  let session_info = exec.execute(env, RISC0_GUEST_ELF)?;
  let block_hash_duration = start.elapsed();

  let block_hash_cycles = session_info.cycles();
  println!(
    "BlockHash verification: {:.2}s, {} cycles, {} segments",
    block_hash_duration.as_secs_f64(),
    block_hash_cycles,
    session_info.segments.len()
  );

  // =========================================================================
  // Run FullHeader verification mode
  // =========================================================================
  println!();
  println!("=== Mode 2: Full Header Verification ===");

  let full_header_bytes = input.to_bytes()?;
  println!("FullHeader input: {} bytes", full_header_bytes.len());

  let env = ExecutorEnv::builder()
    .segment_limit_po2(MAX_PO2)
    .write(&VerifyMode::FullHeader)?
    .write_frame(&full_header_bytes)
    .build()?;

  let exec = default_executor();
  println!("Executing...");
  let start = std::time::Instant::now();
  let session_info = exec.execute(env, RISC0_GUEST_ELF)?;
  let full_header_duration = start.elapsed();

  let full_header_cycles = session_info.cycles();
  println!(
    "FullHeader verification: {:.2}s, {} cycles, {} segments",
    full_header_duration.as_secs_f64(),
    full_header_cycles,
    session_info.segments.len()
  );

  // =========================================================================
  // Summary
  // =========================================================================
  println!();
  println!("=== Summary for block {} ===", block_number);
  println!("Block hash: {}", block_hash);
  println!();
  println!(
    "  BlockHash mode:   {:>12} cycles  ({:.2}s)",
    block_hash_cycles,
    block_hash_duration.as_secs_f64()
  );
  println!(
    "  FullHeader mode:  {:>12} cycles  ({:.2}s)",
    full_header_cycles,
    full_header_duration.as_secs_f64()
  );
  println!();
  let diff = full_header_cycles as i64 - block_hash_cycles as i64;
  let diff_pct = (diff as f64 / block_hash_cycles as f64) * 100.0;
  println!(
    "  Difference:       {:>+12} cycles  ({:+.1}%)",
    diff, diff_pct
  );
  println!();
  println!("Both verification modes passed successfully!");

  Ok(())
}
