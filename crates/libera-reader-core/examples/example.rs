use anyhow::Result;
use libera_reader_core::ctx::Ctx;
use mimalloc::MiMalloc;
use rfd::AsyncFileDialog;
use std::thread::sleep;
use std::time::Duration;

#[global_allocator]
static GLOBAL_ALLOCATOR: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
  better_panic::install();
  utils::create_subscriber()?;
  let mut ctx = Ctx::new();
  let path_to_scan_is_some = ctx.settings.read().path_to_scan.is_some();
  match path_to_scan_is_some {
    true => {
      ctx.services.run().await?;
    }
    false => {
      println!("Please input path to scan:");
      if let Some(folder) = AsyncFileDialog::new().pick_folder().await {
        let path = folder.path().to_path_buf();
        ctx.settings.set_path_to_scan(path)?;
        ctx.services.run().await?;
      }
    }
  }
  loop {
    sleep(Duration::from_secs(10));
  }
}
