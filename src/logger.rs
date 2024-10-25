use colored::Colorize;
use env_logger::Builder;
use log::{Level, LevelFilter};
use std::io::Write;

pub fn init() {
  Builder::new()
    // @TODO expose cli and rcfile options for log level
    .filter_level(LevelFilter::Debug)
    .format(|buf, record| {
      match record.level() {
        // Normal output shown to users
        Level::Info => {
          writeln!(buf, "{}", record.args())
        }
        Level::Error => {
          writeln!(buf, "{}", format!("✗ {}", record.args()).red())
        }
        Level::Warn => {
          writeln!(buf, "{}", format!("! {}", record.args()).yellow())
        }
        Level::Debug => {
          writeln!(buf, "{}", format!("? {}", record.args()).cyan())
        }
        Level::Trace => {
          writeln!(buf, "{}", record.args())
        }
      }
    })
    .init();
}
