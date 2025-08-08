use anyhow::Result;
use libera_reader_core::ctx::Ctx;
use std::io;
use std::thread::sleep;
use std::time::Duration;


fn main() -> Result<()> {
  utils::create_subscriber()?;
  let mut ctx = Ctx::new()?;
  let path_to_scan_is_some = ctx.settings.read().path_to_scan.is_some();
  match path_to_scan_is_some {
    true => { ctx.services.run()?; }
    false => {
      println!("Please input path to scan:");
      let mut user_input = String::new();
      io::stdin().read_line(&mut user_input).expect("Error: unable to read user input");
      let user_input = user_input.trim().to_string();
      ctx.settings.set_path_to_scan(user_input.clone())?;
      ctx.services.run()?;
    }
  }
  loop {
    sleep(Duration::from_secs(10));
  }
}
