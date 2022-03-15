//! Update version information in project files

mod error;
mod json_node_extra;

pub use error::ScriptError;
pub use json_node_extra::*;

use chrono::prelude::*;
use core::fmt::Arguments;
use evalexpr::*;
use json5_nodes::JsonNode;
use regex::{Captures, RegexBuilder};
use std::borrow::Cow;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use clap::{AppSettings, Parser};
#[derive(Parser)]
#[clap(version, about, long_about = None)]
#[clap(global_setting(AppSettings::NoAutoHelp))]
#[clap(global_setting(AppSettings::NoAutoVersion))]
struct Cli {
  /// The versioning operation to perform
  operation: Option<String>,

  /// Specify the version file explicitly
  #[clap(short, long = "input", parse(from_os_str), value_name = "INPUT_FILE")]
  input_file: Option<PathBuf>,

  /// Actually do the update
  #[clap(short, long)]
  update: bool,
}

pub trait StampVerLog {
  fn output(self: &Self, args: Arguments);
  fn warning(self: &Self, args: Arguments);
  fn error(self: &Self, args: Arguments);
}

#[macro_export]
macro_rules! output {
  ($log: expr, $fmt: expr) => {
    $log.output(format_args!($fmt))
  };
  ($log: expr, $fmt: expr, $($args: tt)+) => {
    $log.output(format_args!($fmt, $($args)+))
  };
}
#[macro_export]
macro_rules! warning {
  ($log: expr, $fmt: expr) => {
    $log.warning(format_args!($fmt))
  };
  ($log: expr, $fmt: expr, $($args: tt)+) => {
    $log.warning(format_args!($fmt, $($args)+))
  };
}

#[macro_export]
macro_rules! error {
  ($log: expr, $fmt: expr) => {
    $log.error(format_args!($fmt))
  };
  ($log: expr, $fmt: expr, $($args: tt)+) => {
    $log.error(format_args!($fmt, $($args)+))
  };
}

pub struct StampVerTool<'a> {
  log: &'a dyn StampVerLog,
}

impl<'a> StampVerTool<'a> {
  pub fn new(log: &'a dyn StampVerLog) -> StampVerTool {
    StampVerTool { log }
  }

  pub fn run(
    self: &mut Self,
    args: impl IntoIterator<Item = std::ffi::OsString>,
  ) -> Result<(), Box<dyn Error>> {
    use clap::IntoApp;
    let matches = Cli::command().try_get_matches_from(args)?;

    if matches.is_present("version") {
      output!(self.log, "{}", Cli::command().get_version().unwrap_or(""));
      return Ok(());
    }

    if matches.is_present("help") {
      let mut output = Vec::new();
      let mut cmd = Cli::command();

      cmd.write_help(&mut output).unwrap();

      output!(self.log, "{}", String::from_utf8(output).unwrap());
      return Ok(());
    }

    use clap::FromArgMatches;
    let cli = Cli::from_arg_matches(&matches)?;

    let (content, root_node, script_file) = self.read_script_file(cli.input_file)?;

    let inner_run = || {
      self.validate_script_file(&root_node)?;

      let mut run_context = self.create_run_context(&root_node)?;

      self
        .run_operation(cli.operation, &root_node, &mut run_context)
        .map_err(|e| ScriptError::new(e.message, Some(script_file.clone()), e.location))?;

      self.process_targets(&script_file, &root_node, cli.update, &mut run_context)?;

      self.update_script_file(&script_file, content, &root_node, &run_context, cli.update)?;

      Ok::<_, ScriptError>(())
    };

    inner_run().map_err(|e| ScriptError::new(e.message, Some(script_file.clone()), e.location))?;

    Ok(())
  }

  fn read_script_file(
    self: &Self,
    input_file: Option<PathBuf>,
  ) -> Result<(String, JsonNode, PathBuf), Box<dyn Error>> {
    let script_file = match input_file {
      Some(input_file) => input_file.canonicalize()?,
      None => {
        // Search for the nearest version file
        match WalkDir::new(".")
          .follow_links(false)
          .into_iter()
          .filter_map(|e| e.ok())
          .filter(|e| e.file_name().to_string_lossy() == "version.json5")
          .min_by_key(|e| e.path().components().count())
        {
          None => {
            return Err(From::from(format!(
              "No 'version.json5' file found in sub-directories"
            )))
          }
          Some(entry) => entry.path().to_owned(),
        }
      }
    };

    output!(self.log, "Using script file '{}'", script_file.display());

    let content = fs::read_to_string(&script_file)?;
    let root_node = json5_nodes::parse(&content)?;

    Ok((content, root_node, script_file))
  }

  fn validate_script_file(self: &Self, root_node: &'a JsonNode) -> Result<(), ScriptError> {
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

  fn create_run_context(self: &Self, root_node: &JsonNode) -> Result<HashMapContext, ScriptError> {
    let mut context = HashMapContext::new();

    // Add all fixed vars into the context
    for (identifier, var_node) in root_node.get_object_entry("vars")?.get_object_iter()? {
      context.set_value(identifier.to_string(), Value::from(var_node.get_value()))?;
    }

    let now: DateTime<Utc> = Utc::now();

    context.set_value(
      "now::year".to_owned(),
      Value::from(i64::from(now.date().year())),
    )?;
    context.set_value(
      "now::month".to_owned(),
      Value::from(i64::from(now.date().month())),
    )?;
    context.set_value(
      "now::day".to_owned(),
      Value::from(i64::from(now.date().day())),
    )?;
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
    for (identifier, calc_var_node) in root_node.get_object_entry("calcVars")?.get_object_iter()? {
      context.set_value(
        identifier.to_string(),
        evalexpr::eval_with_context(&calc_var_node.get_string(), &context)
          .map_err(|e| script_error!(e.to_string(), calc_var_node))?,
      )?;
    }

    if let None = context.get_value("tz") {
      fn get_local_tz_iana_name() -> Option<String> {
        if let Ok(path) = fs::read_link("/etc/localtime") {
          let path_zoneinfo = OsStr::new("zoneinfo");
          let arr = path
            .iter()
            .skip_while(|s| *s != path_zoneinfo)
            .skip(1)
            .map(|s| s.to_string_lossy().to_string())
            .collect::<Vec<_>>();

          Some(format!("{}/{}", arr[0], arr[1]))
        } else {
          None
        }
      }

      if let Some(tz_iana_name) = get_local_tz_iana_name() {
        warning!(
          self.log,
          "No 'tz' timezone value set; using local time zone '{}'",
          tz_iana_name
        );
        context.set_value("tz".to_owned(), Value::from(tz_iana_name))?;
      }
    }

    Ok(context)
  }

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

      output!(self.log, "Operation '{}'", operation);
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

  pub fn process_targets(
    self: &Self,
    script_file: &PathBuf,
    root_node: &JsonNode,
    update: bool,
    context: &mut HashMapContext,
  ) -> Result<(), ScriptError> {
    let version_file_dir = script_file.parent().unwrap_or(Path::new("."));

    for target_node in root_node.get_object_entry("targets")?.get_array_iter()? {
      for target_file_node in target_node.get_object_entry("files")?.get_array_iter()? {
        let target_file = version_file_dir.join(target_file_node.get_string());
        let updates_node = target_node.get_object_entry("updates").ok();
        let write_node = target_node.get_object_entry("write").ok();
        let copy_from_node = target_node.get_object_entry("copyFrom").ok();
        let mut action = "".to_string();

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
              warning!(
                self.log,
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

          ()
        } else if let Some(copy_from_node) = copy_from_node {
          if update {
            let copy_from_str = copy_from_node.get_string();
            let s = eval_string_with_context(&copy_from_str, context)?;
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
          ()
        } else if let Some(write_node) = write_node {
          if update {
            let file_content = write_node.get_string();
            fs::write(
              &target_file,
              eval_string_with_context(&file_content, context)?,
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
          ()
        }

        output!(
          self.log,
          "{} '{}' -> '{}'",
          action,
          target_node.get_object_entry("description")?.get_string(),
          target_file.display().to_string()
        );
      }
    }
    Ok(())
  }

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
          Value::String(s) => format!("\"{}\"", s),
          Value::Float(f) => format!("{}", f),
          Value::Boolean(b) => format!("{}", b),
          Value::Int(n) => format!("{}", n),
          _ => "".to_string(),
        };
        let re = RegexBuilder::new(
          &("(?P<begin>vars:\\s*\\{\n(?:.*\n)*?\\s*".to_string()
            + &identifier
            + "\\s*:\\s).*?(?P<end>\\s*,.*?\n)"),
        )
        .multi_line(true)
        .build()
        .map_err(|_| script_error!(format!("Unable to replace var '{}'", identifier), var_node))?;

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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_run() {
    let temp_dir = tempfile::tempdir().unwrap();
    let version_file = temp_dir.path().join("version.json5");
    let version_content = r##"// Test file
  {
    vars: {
      major: 3,
      minor: 0,
      patch: 0,
      build: 20210902,
      revision: 0,
      tz: "America/Los_Angeles",
      sequence: 6,
      buildType: "test",
      pi: 3.14,
      debug: true,
    },
    calcVars: {
      nextBuild: "now::year * 10000 + now::month * 100 + now::day",
      nextSequence: "sequence + 1",
    },
    operations: {
      incrMajor: "major += 1; minor = 0; patch = 0; revision = 0; build = nextBuild",
      incrMinor: "minor += 1; patch = 0; revision = 0; build = nextBuild",
      incrPatch: "patch += 1; revision = 0; build = nextBuild",
      incrRevision: "revision += 1; build = nextBuild",
      incrSequence: "sequence += 1",
      setBetaBuild: 'buildType = "beta"',
      setProdBuild: 'buildType = "prod"',
    },
    targets: [
      {
        description: "NodeJS package file",
        files: ["package.json"],
        updates: [
          {
            search: '^(?P<begin>\\s*"version"\\s*:\\s*")\\d+\\.\\d+\\.\\d+(?P<end>")',
            replace: 'begin + str::from(major) + "." + str::from(minor) + "." + str::from(patch) + end',
          },
        ],
      },
      {
        description: "TypeScript version",
        files: ["version.ts"],
        updates: [
          {
            search: '^(?P<begin>\\s*export\\s*const\\s*version\\s*=\\s*")\\d+\\.\\d+\\.\\d+(?P<end>";?)$',
            replace: 'begin + str::from(major) + "." + str::from(minor) + "." + str::from(patch) + end',
          },
          {
            search: '^(?P<begin>\\s*export\\s*const\\s*fullVersion\\s*=\\s*")\\d+\\.\\d+\\.\\d+\\+\\d+\\.\\d+(?P<end>";?)$',
            replace: 'begin + str::from(major) + "." + str::from(minor) + "." + str::from(patch) + "+" + str::from(build) + "." + str::from(revision) + end',
          },
        ],
      },
      {
        description: "Git version tag commit message",
        files: ["version.desc.txt"],
        write: '"Version" + str::from(major) + "." + str::from(minor) + "." + str::from(patch) + "+" + str::from(build) + "." + str::from(revision)',
      },
      {
        description: "Google Firebase",
        files: ["some-file.plist"],
        copyFrom: '"some-file" + if(buildType == "test", "-test", "-prod") + ".plist"',
      },
    ],
  }
          "##;
    let package_json_content = r##"
  {
    "name": "dummy",
    "version": "0.0.0"
  }
      "##;
    let version_ts_content = r##"
  export const version = "10.0.0";
  export const fullVersion = "10.0.0+20210903.0";
      "##;

    fs::write(&version_file, &version_content).unwrap();
    println!("{}", version_file.display());
    fs::write(&temp_dir.path().join("package.json"), &package_json_content).unwrap();
    fs::write(&temp_dir.path().join("version.ts"), &version_ts_content).unwrap();
    fs::write(
      &temp_dir.path().join("some-file-test.plist"),
      "some-file-test",
    )
    .unwrap();

    struct TestLogger;

    impl TestLogger {
      fn new() -> TestLogger {
        TestLogger {}
      }
    }

    impl StampVerLog for TestLogger {
      fn output(self: &Self, _args: Arguments) {}
      fn warning(self: &Self, _args: Arguments) {}
      fn error(self: &Self, _args: Arguments) {}
    }

    let logger = TestLogger::new();
    let mut tool = StampVerTool::new(&logger);
    let args1: Vec<std::ffi::OsString> = vec![
      "".into(),
      "incrMajor".into(),
      "-i".into(),
      version_file.display().to_string().into(),
    ];

    tool.run(args1).unwrap();

    let args2: Vec<std::ffi::OsString> = vec![
      "".into(),
      "incrMajor".into(),
      "-i".into(),
      version_file.display().to_string().into(),
      "-u".into(),
    ];

    tool.run(args2).unwrap();
  }
}
