use evalexpr::*;
use json5_nodes::{JsonError, Location};
use std::convert::From;
use std::error::Error;
use std::fmt::{self, Display};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
/// Represents an error that occurred during script execution.
pub struct ScriptError {
    /// The error message.
    pub message: String,
    /// The file path of the script where the error occurred.
    pub script_file: Option<PathBuf>,
    /// The location within the script where the error occurred.
    pub location: Option<Location>,
}

impl ScriptError {
    /// Create a new ScriptError instance.
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
/// Create a new ScriptError instance.
macro_rules! script_error {
    ($msg: expr, $node: expr) => {
        ScriptError::new($msg.to_string(), None, $node.get_location())
    };
}
