use colored::Colorize;
use env_logger::Builder;
use log::{Level, LevelFilter};
use std::io::Write;

/// Initialize the logger with the given log level
pub fn init(log_levels: &[LevelFilter]) {
  if !log_levels.contains(&LevelFilter::Off) {
    let error_enabled = log_levels.contains(&LevelFilter::Error);
    let warn_enabled = log_levels.contains(&LevelFilter::Warn);
    let info_enabled = log_levels.contains(&LevelFilter::Info);
    let debug_enabled = log_levels.contains(&LevelFilter::Debug);
    let trace_enabled = log_levels.contains(&LevelFilter::Trace);
    Builder::new()
      .filter_level(LevelFilter::Debug)
      .format(move |buf, record| {
        let level = record.level();
        if level == Level::Error && error_enabled {
          writeln!(buf, "{}", format!("✗ {}", record.args()).red())
        } else if level == Level::Warn && warn_enabled {
          writeln!(buf, "{}", format!("! {}", record.args()).yellow())
        } else if level == Level::Info && info_enabled {
          writeln!(buf, "{}", record.args())
        } else if level == Level::Debug && debug_enabled {
          writeln!(buf, "{}", format!("? {}", record.args()).cyan())
        } else if level == Level::Trace && trace_enabled {
          writeln!(buf, "{}", record.args())
        } else {
          write!(buf, "")
        }
      })
      .init();
  }
}
