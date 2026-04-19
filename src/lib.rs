//! Version stamping tool
//!
//! Update version information in project files
//!
//! This is a library and command line tool for version stamping.
//!
#![deny(unsafe_code, missing_docs)]

mod error;
mod json_node_extra;

pub use error::ScriptError;

use anyhow::Context as AnyhowContext;
use evalexpr::*;
use jiff::{Zoned, tz::TimeZone};
use json_node_extra::*;
use json5_nodes::JsonNode;
use regex::{Captures, RegexBuilder};
use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
};

/// Versioning tool for stamping version information into files.
pub struct StampVerTool {}

impl StampVerTool {
    /// Create a new instance of StampVerTool.
    pub fn new() -> StampVerTool {
        StampVerTool {}
    }

    /// Read the script file and return its content and root node.
    pub fn read_script_file(
        self: &Self,
        input_file: PathBuf,
    ) -> anyhow::Result<(String, JsonNode, PathBuf)> {
        let script_path = input_file.canonicalize()?;
        let content = fs::read_to_string(&script_path)?;
        let root_node = json5_nodes::parse(&content)?;

        Ok((content, root_node, script_path))
    }

    /// Validate the filter path.
    pub fn validate_filter_path(
        self: &Self,
        filter_path: Option<&Path>,
    ) -> anyhow::Result<PathBuf> {
        match filter_path {
            Some(path) => {
                let mut path = path.to_path_buf();

                if path.is_relative() {
                    path = std::env::current_dir()?.join(path);
                }

                path = path.canonicalize().context(format!(
                    "Failed to canonicalize filter path '{}'",
                    path.display()
                ))?;

                if !path.is_dir() {
                    return Err(anyhow::anyhow!(
                        "Filter path '{}' is not a directory",
                        path.display()
                    ));
                }

                Ok(path)
            }
            None => Ok(std::env::current_dir()?),
        }
    }

    /// Validate the script file's root node.
    pub fn validate_script_file(self: &Self, root_node: &JsonNode) -> Result<(), ScriptError> {
        if !root_node.is_object() {
            return Err(script_error!("Node <root> is not an object", root_node));
        }

        // vars node
        let vars_node = root_node.get_object_entry("vars")?;
        let vars_iter = vars_node.get_object_iter()?;

        for (key, var_node) in vars_iter {
            if key == "tz" && !var_node.is_string() {
                return Err(script_error!("'tz' node must be a string", var_node));
            } else if !(var_node.is_string()
                || var_node.is_integer()
                || var_node.is_float()
                || var_node.is_bool())
            {
                return Err(script_error!(
                    format!("'vars' entry must be a string, integer, float or boolean",),
                    var_node
                ));
            }
        }

        if let Ok(calc_vars_node) = root_node.get_object_entry("calcVars") {
            let calc_vars_iter = calc_vars_node.get_object_iter()?;

            for (key, node) in calc_vars_iter {
                if !node.is_string() {
                    return Err(script_error!(
                        format!("'calcVars' entry '{}' must be a string", key),
                        calc_vars_node
                    ));
                }
            }
        }

        let operations_node = root_node.get_object_entry("operations")?;
        let operations_iter = operations_node.get_object_iter()?;

        for (key, operation_node) in operations_iter {
            if !operation_node.is_string() {
                return Err(script_error!(
                    format!("Operation '{}' must be a string", key),
                    operation_node
                ));
            }
        }

        let targets_node = root_node.get_object_entry("targets")?;
        let targets_iter = targets_node.get_array_iter()?;

        if let None = targets_iter.clone().nth(0) {
            return Err(script_error!("'targets' must not be empty", targets_node));
        }

        for (index, target_node) in targets_iter.enumerate() {
            if !target_node.is_object() {
                return Err(script_error!(
                    format!("'targets' entry '{}' must be an object", index),
                    target_node
                ));
            }

            let description_node = target_node.get_object_entry("description")?;

            if !description_node.is_string() {
                return Err(script_error!(
                    "'description' entry must be a string",
                    description_node
                ));
            }

            let files_node = target_node.get_object_entry("files")?;
            let mut files_iter = files_node.get_array_iter()?;

            if let None = files_iter.nth(0) {
                return Err(script_error!("'files' must not be empty", files_node));
            }

            let updates_node = target_node.get_object_entry("updates").ok();
            let write_node = target_node.get_object_entry("write").ok();
            let copy_from_node = target_node.get_object_entry("copyFrom").ok();

            if let Some(updates_node) = updates_node {
                let updates_iter = updates_node.get_array_iter()?;

                if let None = updates_iter.clone().nth(0) {
                    return Err(script_error!("'updates' must not be empty", updates_node));
                }

                for (index, item_node) in updates_iter.enumerate() {
                    if !item_node.is_object() {
                        return Err(script_error!(
                            format!("'updates' entry '{}' must be an object", index),
                            target_node
                        ));
                    }

                    let search_node = item_node.get_object_entry("search")?;
                    let replace_node = item_node.get_object_entry("replace")?;

                    if !search_node.is_string() {
                        return Err(script_error!("'search' entry must be string", search_node));
                    }

                    if !replace_node.is_string() {
                        return Err(script_error!(
                            "'replace' entry must be string",
                            replace_node
                        ));
                    }
                }
            } else if let Some(write_node) = write_node {
                if !write_node.is_string() {
                    return Err(script_error!("'write' entry must be string", write_node));
                }
            } else if let Some(copy_from_node) = copy_from_node {
                if !copy_from_node.is_string() {
                    return Err(script_error!(
                        "'copyFrom' entry must be string",
                        copy_from_node
                    ));
                }
            } else {
                return Err(script_error!(
                    "Target must contain 'updates', 'write' or 'copyFrom'",
                    target_node
                ));
            }
        }

        Ok(())
    }

    /// Create a run context from the root node.
    pub fn create_run_context(
        self: &Self,
        root_node: &JsonNode,
    ) -> Result<HashMapContext, ScriptError> {
        let mut context = HashMapContext::new();

        // Add all fixed vars into the context
        for (identifier, var_node) in root_node.get_object_entry("vars")?.get_object_iter()? {
            context.set_value(identifier.to_string(), Value::from(var_node.get_value()))?;
        }

        let tz: TimeZone;

        if let Some(Value::String(tz_value)) = context.get_value("tz") {
            let iana_name = tz_value.as_str();
            tz = TimeZone::get(iana_name).map_err(|e| script_error!(e.to_string()))?;
        } else {
            tz = TimeZone::system();
            log::warn!(
                "'tz' value not set or not a string; using system time zone '{}'",
                tz.iana_name().unwrap()
            );
        }

        context.set_value("tz".to_owned(), Value::from(tz.iana_name().unwrap()))?;

        let now: Zoned = Zoned::now();

        now.with_time_zone(tz);

        context.set_value("now::year".to_owned(), Value::Int(i64::from(now.year())))?;
        context.set_value("now::month".to_owned(), Value::Int(i64::from(now.month())))?;
        context.set_value("now::day".to_owned(), Value::Int(i64::from(now.day())))?;
        context.set_function(
            "if".to_owned(),
            Function::new(|arg| {
                if let Ok(tuple) = arg.as_tuple() {
                    if let Value::Boolean(b) = tuple[0] {
                        if b {
                            Ok(tuple[1].clone())
                        } else {
                            Ok(tuple[2].clone())
                        }
                    } else {
                        Err(EvalexprError::expected_boolean(tuple[0].clone()))
                    }
                } else {
                    Err(EvalexprError::expected_tuple(arg.clone()))
                }
            }),
        )?;

        // Evaluate the calculated vars
        for (identifier, calc_var_node) in
            root_node.get_object_entry("calcVars")?.get_object_iter()?
        {
            let value = evalexpr::eval_with_context(&calc_var_node.get_string(), &context)
                .map_err(|e| script_error!(e.to_string(), calc_var_node))?;

            context.set_value(identifier.to_owned(), value)?;
        }

        Ok(context)
    }

    /// Run an operation from the script file.
    pub fn run_operation(
        self: &Self,
        operation: Option<String>,
        root_node: &JsonNode,
        context: &mut HashMapContext,
    ) -> Result<(), ScriptError> {
        let operations_node = root_node.get_object_entry("operations")?;

        if let Some(operation) = operation {
            let operation_node = operations_node.get_object_entry(&operation).map_err(|_| {
                script_error!(
                    format!("Operation '{}' not found", operation),
                    operations_node
                )
            })?;

            log::info!("Operation '{}'", operation);
            evalexpr::eval_with_context_mut(&operation_node.get_string(), context)
                .map_err(|e| script_error!(e.to_string(), operation_node))?;
            Ok(())
        } else {
            Err(script_error!(
                format!(
                    "Specify a valid operation, one of {}",
                    operations_node
                        .get_object_iter()?
                        .map(|(identifier, _)| format!("'{}'", identifier))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                operations_node
            ))
        }
    }

    /// Process the targets defined in the script file.
    pub fn process_targets(
        self: &Self,
        script_file: &PathBuf,
        root_node: &JsonNode,
        update: bool,
        context: &mut HashMapContext,
        filter_path: &Path,
    ) -> Result<(), ScriptError> {
        let version_file_dir = script_file.parent().unwrap_or(Path::new("."));

        for target_node in root_node.get_object_entry("targets")?.get_array_iter()? {
            for target_file_node in target_node.get_object_entry("files")?.get_array_iter()? {
                let updates_node = target_node.get_object_entry("updates").ok();
                let write_node = target_node.get_object_entry("write").ok();
                let copy_from_node = target_node.get_object_entry("copyFrom").ok();
                let mut action = "".to_string();
                let mut target_file = PathBuf::from(target_file_node.get_string());

                if target_file.is_absolute() {
                    return Err(script_error!(
                        format!(
                            "Target file '{}' is an absolute path, but should be relative to the script file",
                            target_file.display().to_string()
                        ),
                        target_file_node
                    ));
                }

                target_file = path_clean::clean(version_file_dir.join(target_file));

                if !target_file.starts_with(filter_path) {
                    log::warn!(
                        "File '{}' is outside the filter path and will be skipped",
                        target_file.display().to_string()
                    );
                    continue;
                }

                if let Some(updates_node) = updates_node {
                    let mut content = fs::read_to_string(&target_file).map_err(|_| {
                        script_error!(
                            format!(
                                "File '{}' does not exist or is not readable",
                                target_file.display().to_string()
                            ),
                            target_file_node
                        )
                    })?;

                    for replacement_node in updates_node.get_array_iter()? {
                        let search_node = replacement_node.get_object_entry("search")?;
                        let search_str = search_node.get_string();
                        let re = RegexBuilder::new(&search_str)
                            .multi_line(true)
                            .build()
                            .map_err(|e| {
                                script_error!(
                                    format!("Regex is not valid - {}", e.to_string()),
                                    search_node
                                )
                            })?;
                        let replace_node = replacement_node.get_object_entry("replace")?;
                        let replace_str = &replace_node.get_string();
                        let mut found = false;
                        let mut replace_err: Option<EvalexprError> = None;

                        content = re
                            .replace_all(&content, |caps: &Captures| {
                                found = true;

                                if let Some(m) = caps.name("begin") {
                                    context
                                        .set_value("begin".to_owned(), Value::from(m.as_str()))
                                        .unwrap();
                                }
                                if let Some(m) = caps.name("end") {
                                    context
                                        .set_value("end".to_owned(), Value::from(m.as_str()))
                                        .unwrap();
                                }
                                let result = eval_string_with_context(replace_str, context);

                                match result {
                                    Ok(s) => s,
                                    Err(_) => {
                                        replace_err = result.err();
                                        String::new()
                                    }
                                }
                            })
                            .into_owned();

                        if let Some(err) = replace_err {
                            return Err(script_error!(err.to_string(), replace_node));
                        }

                        if !found {
                            log::warn!(
                                "Search/replace in '{}' did not match anything; check your search string '{}'",
                                target_file.display().to_string(),
                                search_str
                            )
                        }
                    }

                    if update {
                        fs::write(&target_file, content).map_err(|_| {
                            script_error!(
                                format!(
                                    "Unable to write to file '{}'",
                                    target_file.display().to_string()
                                ),
                                target_file_node
                            )
                        })?;
                        action += "Updated";
                    } else {
                        action += "Would update";
                    }
                } else if let Some(copy_from_node) = copy_from_node {
                    if update {
                        let copy_from_str = copy_from_node.get_string();
                        let s = eval_string_with_context(&copy_from_str, context)
                            .map_err(|e| script_error!(e.to_string(), copy_from_node))?;
                        let from_file = version_file_dir.join(s);

                        fs::copy(&from_file, &target_file).map_err(|_| {
                            script_error!(
                                format!(
                                    "unable to copy {} to {}",
                                    from_file.display().to_string(),
                                    target_file.display().to_string(),
                                ),
                                copy_from_node
                            )
                        })?;
                        action += "Copied";
                    } else {
                        action += "Would copy"
                    }
                } else if let Some(write_node) = write_node {
                    if update {
                        let file_content = write_node.get_string();

                        fs::write(
                            &target_file,
                            eval_string_with_context(&file_content, context)
                                .map_err(|e| script_error!(e.to_string(), write_node))?,
                        )
                        .map_err(|_| {
                            script_error!(
                                format!("Unable to write '{}'", target_file.display().to_string(),),
                                write_node
                            )
                        })?;
                        action += "Wrote";
                    } else {
                        action += "Would write";
                    }
                }

                log::info!(
                    "{} '{}' -> '{}'",
                    action,
                    target_node.get_object_entry("description")?.get_string(),
                    target_file.display().to_string()
                );
            }
        }
        Ok(())
    }

    /// Update the script file with the given content and root node.
    pub fn update_script_file(
        self: &Self,
        script_file: &Path,
        content: String,
        root_node: &JsonNode,
        run_context: &HashMapContext,
        update: bool,
    ) -> Result<(), ScriptError> {
        let mut new_content = Cow::from(content).into_owned();
        let vars_node = root_node.get_object_entry("vars")?;

        for (identifier, var_node) in vars_node.get_object_iter()? {
            if let Some(value) = run_context.get_value(identifier) {
                let s = match value {
                    Value::String(s) => {
                        if s.contains("\"") {
                            format!("'{}'", s)
                        } else {
                            format!("\"{}\"", s)
                        }
                    }
                    Value::Float(f) => format!("{}", f),
                    Value::Boolean(b) => format!("{}", b),
                    Value::Int(n) => format!("{}", n),
                    _ => "".to_string(),
                };
                let re = RegexBuilder::new(
                    &("(?P<begin>vars:\\s*\\{\n(?:.*\n)*?\\s*".to_string()
                        + &identifier
                        + "\\s*:\\s*).*?(?P<end>\\s*,?\\s*?\n)"),
                )
                .multi_line(true)
                .build()
                .map_err(|_| {
                    script_error!(format!("Unable to replace var '{}'", identifier), var_node)
                })?;

                new_content = re
                    .replace(&new_content, "${begin}".to_string() + &s + "${end}")
                    .into_owned();
            }
        }

        if update {
            fs::write(&script_file, &new_content)
                .map_err(|err| script_error!(err.to_string(), root_node))?;
        }

        Ok(())
    }
}
