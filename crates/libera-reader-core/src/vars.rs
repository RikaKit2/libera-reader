use std::sync::atomic::AtomicBool;


pub static SHUTDOWN: AtomicBool = AtomicBool::new(false);
