use colored::*;
use std::fmt;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{
  fmt::{FmtContext, FormatEvent, FormatFields, format},
  registry::LookupSpan,
};

pub struct Formatter {
  pub show_location: bool,
}

impl<S, N> FormatEvent<S, N> for Formatter
where
  S: Subscriber + for<'a> LookupSpan<'a>,
  N: for<'a> FormatFields<'a> + 'static,
{
  fn format_event(
    &self, _ctx: &FmtContext<'_, S, N>, mut writer: format::Writer<'_>, event: &Event<'_>,
  ) -> fmt::Result {
    let metadata = event.metadata();

    let mut msg = String::new();
    let mut visitor = StringVisitor(&mut msg);
    event.record(&mut visitor);

    let colored_msg = match metadata.target() {
      "title" => msg.cyan().bold().to_string(),
      "timing" => msg.bright_black().to_string(),
      "debug" => msg.blue().to_string(),
      "error" => msg.red().bold().to_string(),
      _ => match *metadata.level() {
        Level::WARN => msg.yellow().to_string(),
        _ => msg.green().to_string(),
      },
    };

    if self.show_location {
      let file = metadata.file().unwrap_or("?");
      let line = metadata.line().unwrap_or(0);
      writeln!(writer, "[{}:{}] {}\n", file, line, colored_msg)
    } else {
      writeln!(writer, "{}\n", colored_msg)
    }
  }
}

struct StringVisitor<'a>(&'a mut String);

impl tracing::field::Visit for StringVisitor<'_> {
  fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
    if field.name() == "message" {
      self.0.push_str(value);
    }
  }
  fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {
    if field.name() == "message" {
      self.0.push_str(&format!("{:?}", value));
    }
  }
}

#[macro_export]
macro_rules! title {
    ($($arg:tt)*) => { tracing::info!(target: "title", $($arg)*) };
}

#[macro_export]
macro_rules! timing {
    ($($arg:tt)*) => { tracing::warning!(target: "timing", $($arg)*) };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => { tracing::debug!(target: "debug", $($arg)*) };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => { tracing::error!(target: "error", $($arg)*) };
}
