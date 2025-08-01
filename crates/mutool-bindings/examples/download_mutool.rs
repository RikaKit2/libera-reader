use anyhow::Result;
use mutool_bindings::download_mutool;
use std::io;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[tokio::main]
async fn main() -> Result<()> {
  let progress = Arc::new(RwLock::new(0.0));
  println!("Please input path to target dir:");
  let mut target_dir = String::new();
  io::stdin().read_line(&mut target_dir).expect("Error: unable to read user input");
  let target_dir = PathBuf::from(target_dir.trim().to_string());
  println!("Please input url to mupdf:");
  let mut url = String::new();
  io::stdin().read_line(&mut url).expect("Error: unable to read user input");
  let url = url.trim().to_string();

  let progress_clone = Arc::clone(&progress);

  tokio::spawn(async move {
    loop {
      let p = *progress_clone.read().unwrap();
      println!("Progress: {:.2}%", p * 100.0);
      if p >= 1.0 {
        break;
      }
      tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
  });

  download_mutool(&url, &target_dir, progress.clone()).await?;
  println!("✅ mutool.exe load in {:?}", target_dir.join("mutool.exe"));
  Ok(())
}
