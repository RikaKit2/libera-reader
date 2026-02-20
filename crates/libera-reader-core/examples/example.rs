use anyhow::Result;
use libera_reader_core::ctx::Ctx;
use mimalloc::MiMalloc;
use std::io;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use utils::error;

#[global_allocator]
static GLOBAL_ALLOCATOR: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
  utils::create_debug_subscriber()?;
  let mut ctx = Ctx::new()?;
  let path_to_scan_is_some = ctx.settings.read().path_to_scan.is_some();
  match path_to_scan_is_some {
    true => {
      ctx.services.run().await?;
    }
    false => {
      set_user_input(&mut ctx).await?;
    }
  }
  loop {
    sleep(Duration::from_secs(10));
  }
}
async fn set_user_input(ctx: &mut Ctx) -> Result<()> {
  loop {
    println!("Please input path to scan:");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).expect("Error: unable to read user input");
    let user_input = user_input.trim().to_string();
    match PathBuf::from(&user_input).is_dir() {
      true => {
        ctx.settings.set_path_to_scan(user_input.clone())?;
        ctx.services.run().await?;
        break;
      }
      false => {
        error!("Entered path to scan is not a directory, enter the path to the directory");
      }
    };
  }
  Ok(())
}
