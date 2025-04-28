// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
  let app = AppWindow::new().unwrap();
  let app_weak = app.as_weak();

  let thread = std::thread::spawn(move || {
    let app_copy = app_weak.clone();
    //Expand the slint window from event loop
    slint::invoke_from_event_loop(move || app_copy.unwrap().window().set_maximized(true)).unwrap();

    //Another code that we wanted to execute after the application was launched
    //For example: hide the console window peculiar to slint
  });

  thread.join().unwrap();
  app.run().unwrap();
  Ok(())
}
