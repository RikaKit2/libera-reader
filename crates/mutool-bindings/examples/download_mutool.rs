use anyhow::Result;
use std::io;
use std::path::PathBuf;
use mutool_bindings::download_mutool_if_missing_blocking;

#[tokio::main]
async fn main() -> Result<()> {
  println!("Please input path to target dir:");
  let mut target_dir = String::new();
  io::stdin().read_line(&mut target_dir).expect("Error: unable to read user input");
  let target_dir = PathBuf::from(target_dir.trim().to_string());
  download_mutool_if_missing_blocking(&target_dir).await?;
  Ok(())
}
