use evalexpr::*;
use json5_nodes::{JsonError, Location};
use std::convert::From;
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

impl From<JsonError> for ScriptError {
  fn from(err: JsonError) -> Self {
    match err {
      JsonError::Syntax(_, location)
      | JsonError::NumberFormat(location)
      | JsonError::NumberRange(location)
      | JsonError::Unicode(location) => ScriptError::new(err.to_string(), None, location),
    }
  }
}

impl From<EvalexprError> for ScriptError {
  fn from(err: EvalexprError) -> Self {
    ScriptError::new(err.to_string(), None, None)
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

#[macro_export]
macro_rules! script_error {
  ($msg: expr, $node: expr) => {
    ScriptError::new($msg.to_string(), None, $node.get_location())
  };
}
