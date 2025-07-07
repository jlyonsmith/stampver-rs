use super::error::ScriptError;
use crate::script_error;
use evalexpr::Value;
use json5_nodes::{Iter, JsonNode, Location};

#[allow(dead_code)]

pub(crate) trait JsonNodeExtra {
    fn is_null(&self) -> bool;
    fn is_bool(&self) -> bool;
    fn is_integer(&self) -> bool;
    fn is_float(&self) -> bool;
    fn is_string(&self) -> bool;
    fn is_array(&self) -> bool;
    fn is_object(&self) -> bool;
    fn get_location(self: &Self) -> Option<Location>;
    fn get_object_entry<'a>(self: &'a Self, name: &str) -> Result<&'a JsonNode, ScriptError>;
    fn get_object_iter(self: &Self) -> Result<Iter<String, JsonNode>, ScriptError>;
    fn get_array_iter(self: &Self) -> Result<std::slice::Iter<JsonNode>, ScriptError>;
    fn get_value(self: &Self) -> Value;
    fn get_string(self: &Self) -> String;
}

impl JsonNodeExtra for JsonNode {
    /// Is the node null?
    fn is_null(&self) -> bool {
        if let JsonNode::Null(_) = self {
            true
        } else {
            false
        }
    }

    /// Is the node a boolean?
    fn is_bool(&self) -> bool {
        if let JsonNode::Bool(_, _) = self {
            true
        } else {
            false
        }
    }

    /// Is the node an integer?
    fn is_integer(&self) -> bool {
        if let JsonNode::Integer(_, _) = self {
            true
        } else {
            false
        }
    }

    /// Is the node a float?
    fn is_float(&self) -> bool {
        if let JsonNode::Float(_, _) = self {
            true
        } else {
            false
        }
    }

    /// Is the node a string?
    fn is_string(self: &Self) -> bool {
        if let JsonNode::String(_, _) = self {
            true
        } else {
            false
        }
    }

    /// Is the node an array?
    fn is_array(self: &Self) -> bool {
        if let JsonNode::Array(_, _) = self {
            true
        } else {
            false
        }
    }

    /// Is the node an object?
    fn is_object(self: &Self) -> bool {
        if let JsonNode::Object(_, _) = self {
            true
        } else {
            false
        }
    }

    /// Get the node location
    fn get_location(self: &Self) -> Option<Location> {
        match self {
            JsonNode::Null(location)
            | JsonNode::Bool(_, location)
            | JsonNode::Integer(_, location)
            | JsonNode::Float(_, location)
            | JsonNode::String(_, location)
            | JsonNode::Array(_, location)
            | JsonNode::Object(_, location) => *location,
        }
    }

    /// Get an object node entry
    fn get_object_entry<'a>(self: &'a Self, name: &str) -> Result<&'a JsonNode, ScriptError> {
        if let JsonNode::Object(map, ..) = self {
            if let Some(node) = map.get(name) {
                Ok(node)
            } else {
                Err(script_error!(
                    format!("Object entry '{}' not found", name),
                    self
                ))
            }
        } else {
            Err(script_error!("Not an object", self))
        }
    }

    // Get object node iterator
    fn get_object_iter(self: &Self) -> Result<Iter<String, JsonNode>, ScriptError> {
        if let JsonNode::Object(map, _) = self {
            Ok(map.iter())
        } else {
            Err(script_error!("Not an object", self))
        }
    }

    // Get array node iterator
    fn get_array_iter(self: &Self) -> Result<std::slice::Iter<JsonNode>, ScriptError> {
        if let JsonNode::Array(array, _) = self {
            Ok(array.iter())
        } else {
            Err(script_error!("Not an array", self))
        }
    }

    fn get_value(self: &Self) -> Value {
        match self {
            JsonNode::Null(..) => Value::Empty,
            JsonNode::Integer(value, ..) => Value::Int(*value as i64),
            JsonNode::Float(value, ..) => Value::Float(*value as f64),
            JsonNode::Bool(value, ..) => Value::from(*value),
            JsonNode::String(value, ..) => Value::from((*value).to_owned()),
            _ => Value::Empty,
        }
    }

    fn get_string(self: &Self) -> String {
        match self {
            JsonNode::Null(..) => "null".to_string(),
            JsonNode::Integer(value, ..) => (*value).to_string(),
            JsonNode::Float(value, ..) => (*value).to_string(),
            JsonNode::Bool(value, ..) => (*value).to_string(),
            JsonNode::String(value, ..) => (*value).to_string(),
            _ => String::new(),
        }
    }
}
