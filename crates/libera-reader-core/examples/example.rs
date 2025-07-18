use anyhow::Result;
use libera_reader_core::ctx::Ctx;
use libera_reader_core::db::get_models;
use native_db::Models;
use once_cell::sync::Lazy;
use std::io;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<()> {
  utils::create_subscriber()?;
  pub static MODELS: Lazy<Models> = Lazy::new(|| get_models().unwrap());
  let mut ctx = Ctx::new(&MODELS)?;
  match ctx.settings.path_to_scan.clone() {
    Some(path_to_scan) => { ctx.services.run(path_to_scan)?; }
    None => {
      println!("Please input path to scan:");
      let mut user_input = String::new();
      io::stdin().read_line(&mut user_input).expect("Error: unable to read user input");
      let user_input = user_input.trim().to_string();
      ctx.settings.set_path_to_scan(user_input.clone())?;
      ctx.services.run(user_input)?;
    }
  };
  loop {
    sleep(Duration::from_secs(10));
  }
}
