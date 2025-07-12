use anyhow::Result;
use libera_reader_core::ctx::Ctx;
use libera_reader_core::db::{get_models, models::Settings};
use native_db::Models;
use once_cell::sync::Lazy;
use std::io;
use std::thread::sleep;
use std::time::Duration;

//noinspection RsUnwrap
fn main() -> Result<()> {
  utils::create_subscriber()?;
  pub static MODELS: Lazy<Models> = Lazy::new(|| get_models().unwrap());
  let mut ctx = Ctx::new(&MODELS)?;
  Settings::create_if_not_exist(&ctx.db)?;

  match ctx.settings.read().unwrap().path_to_scan.clone() {
    Some(path_to_scan) => { ctx.services.run(path_to_scan)?; }
    None => {
      println!("Please input path to scan:");
      let mut user_input = String::new();
      io::stdin().read_line(&mut user_input).expect("Error: unable to read user input");
      let user_input = user_input.trim().to_string();
      ctx.settings.write().unwrap().set_path_to_scan(user_input.clone(), &ctx.db)?;
      ctx.services.run(user_input)?;
    }
  };
  loop {
    sleep(Duration::from_secs(10));
  }
}
