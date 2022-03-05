use colored::Colorize;
use core::fmt::Arguments;
use stampver::{StampVerLog, StampVerTool};

struct StampVerLogger;

impl StampVerLogger {
  fn new() -> StampVerLogger {
    StampVerLogger {}
  }
}

impl StampVerLog for StampVerLogger {
  fn output(self: &Self, args: Arguments) {
    println!("{}", args);
  }
  fn warning(self: &Self, args: Arguments) {
    eprintln!("{}", format!("warning: {}", args).yellow());
  }
}

fn main() {
  let logger = StampVerLogger::new();

  if let Err(error) = StampVerTool::new(&logger).run(std::env::args_os()) {
    // TODO: Use the logger here too
    eprint!("{}", error);
    std::process::exit(1);
  }
}
