use json5_nodes::Location;
use std::error::Error;
use std::fmt::{self, Display};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub struct ScriptError {
  pub message: String,
  pub script_file: Option<PathBuf>,
  pub location: Option<Location>,
}

impl ScriptError {
  pub fn new(
    message: String,
    script_file: Option<PathBuf>,
    location: Option<Location>,
  ) -> ScriptError {
    ScriptError {
      message,
      script_file,
      location,
    }
  }
}

impl Display for ScriptError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let Some(file) = &self.script_file {
      if let Some(location) = self.location {
        return write!(
          formatter,
          "{} ({},{}): {}",
          file.to_str().unwrap_or("???"),
          location.line,
          location.column,
          self.message
        );
      }
    }
    write!(formatter, "{}", self.message)
  }
}

impl Error for ScriptError {}

// impl From<ScriptError> for Box<dyn Error> {
//   fn from(err: ScriptError) -> Self {
//     Box::new(err)
//   }
// }
