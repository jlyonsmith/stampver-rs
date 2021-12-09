//! Update version information in project files

use chrono::prelude::*;
use core::fmt::Arguments;
use evalexpr::*;
use json5_nodes::JsonNode;
use regex::{Captures, RegexBuilder};
use std::{collections::HashMap, env::ArgsOs, error::Error, path::PathBuf};

use clap::{AppSettings, Parser};
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
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
}

macro_rules! output {
  ($log: expr, $fmt: expr) => {
    $log.output(format_args!($fmt))
  };
  ($log: expr, $fmt: expr, $($args: tt)+) => {
    $log.output(format_args!($fmt, $($args)+))
  };
}

pub struct StampVerTool<'a> {
  log: &'a dyn StampVerLog,
}

impl<'a> StampVerTool<'a> {
  pub fn new(log: &'a dyn StampVerLog) -> StampVerTool {
    StampVerTool { log }
  }

  pub fn run(self: &Self, args: ArgsOs) -> Result<(), Box<dyn Error>> {
    #[cfg(windows)]
    ansi_term::enable_ansi_support().ok();

    use clap::IntoApp;
    let matches = Cli::into_app().try_get_matches_from(args)?;

    if matches.is_present("version") {
      fn get_version<'a, T: IntoApp>() -> &'a str {
        <T as IntoApp>::into_app().get_version().unwrap_or("")
      }

      output!(self.log, "{}", get_version::<Cli>());
      return Ok(());
    }

    if matches.is_present("help") {
      fn get_help<T: IntoApp>() -> String {
        let mut output = Vec::new();
        <T as IntoApp>::into_app().write_help(&mut output).unwrap();
        let output = String::from_utf8(output).unwrap();
        output
      }

      output!(self.log, "{}", get_help::<Cli>());
      return Ok(());
    }

    use clap::FromArgMatches;
    let cli = Cli::from_arg_matches(&matches)?;

    let (content, root_node, script_file) = self.read_script_file(cli.input_file)?;

    //self.validate_script_file(root_node)?;

    //let (run_context, interpolator) = self.create_run_context(root_node);

    //run_operation(operation, &version_info, &mut context)?;

    //process_targets(script_node, cli.update, &mut run_context)?;

    //update_script_file(&content, root_node, &run_context)?;

    Ok(())
  }

  fn read_script_file(
    self: &Self,
    input_file: Option<PathBuf>,
  ) -> Result<(&str, (), String), Box<dyn Error>> {
    //   let version_file = match input_file {
    //     Some(input_file) => Path::new(input_file).canonicalize()?,
    //     None => {
    //       let mut path = None;

    //       for entry in WalkDir::new(".") {
    //         if let Ok(entry) = entry {
    //           if entry.file_type().is_file()
    //             && entry
    //               .file_name()
    //               .to_str()
    //               .map(|s| s.starts_with("version.json"))
    //               .unwrap_or(false)
    //           {
    //             path = Some(Path::new(entry.path()).to_path_buf());
    //             break;
    //           }
    //         }
    //       }
    //       if path.is_none() {
    //         return Err(From::from(format!("No version.json file found")));
    //       }

    //       path.unwrap()
    //     }
    //   };
    Ok(("", (), String::new()))
  }

  fn create_run_context(node: &JsonNode) -> Result<(), Box<dyn Error>> {
    let mut context = HashMapContext::new();

    //   // Add all fixed vars into the context
    //   for (identifier, value) in version_info.vars.iter() {
    //     context.set_value(identifier.to_string(), value.as_expr_value())?;
    //   }

    //   let now: DateTime<Utc> = Utc::now();

    //   context.set_value(
    //     "now::year".to_owned(),
    //     Value::from(i64::from(now.date().year())),
    //   )?;
    //   context.set_value(
    //     "now::month".to_owned(),
    //     Value::from(i64::from(now.date().month())),
    //   )?;
    //   context.set_value(
    //     "now::day".to_owned(),
    //     Value::from(i64::from(now.date().day())),
    //   )?;
    //   context.set_function(
    //     "if".to_owned(),
    //     Function::new(|arg| {
    //       if let Ok(tuple) = arg.as_tuple() {
    //         if let Value::Boolean(b) = tuple[0] {
    //           if b {
    //             Ok(tuple[1].clone())
    //           } else {
    //             Ok(tuple[2].clone())
    //           }
    //         } else {
    //           Err(EvalexprError::expected_boolean(tuple[0].clone()))
    //         }
    //       } else {
    //         Err(EvalexprError::expected_tuple(arg.clone()))
    //       }
    //     }),
    //   )?;

    //   // Evaluate the calculated vars
    //   for (identifier, value) in version_info.calc_vars.iter() {
    //     context.set_value(
    //       identifier.to_string(),
    //       evalexpr::eval_with_context(&value, &context)?,
    //     )?;
    //   }

    // TODO: Do this when validating the script
    // if let Some(_) = context.get_value("tz") {
    //   warn!(
    //     self.log,
    //     "No 'tz' timezone value set; using local time zone {}", "local-time-zone"
    //   );
    // }

    Ok(())
  }

  pub fn run_operation(operation: &String) {
    //   operation: &str,
    //   version_info: &VersionInfo,
    //   context: &mut HashMapContext,
    // ) -> Result<(), Box<dyn Error>> {
    //   if let Some(value) = version_info.operations.get(operation) {
    //     eprintln!("{} {}", "Operation".bold().green(), operation);
    //     eval_with_context_mut(&value, context)?;
    //   } else {
    //     return Err(From::from(format!(
    //       "No operation named {} was found",
    //       operation.bright_blue()
    //     )));
    //   }
  }

  pub fn process_targets(
    version_file_dir: &PathBuf,
    root_node: &JsonNode,
    update: bool,
    context: &mut HashMapContext,
  ) -> Result<(), Box<dyn Error>> {
    //   for target in version_info.targets.iter() {
    //     for target_file in target.files.iter() {
    //       let target_file = version_file_dir.join(target_file);
    //       let mut action = "".to_string();

    //       match &target.action {
    //         Action::Update(replacements) => {
    //           let mut content = std::fs::read_to_string(&target_file).map_err(|_| {
    //             format!(
    //               "{} does not exist or is not readable",
    //               target_file.display().to_string().bright_blue()
    //             )
    //           })?;

    //           for replacement in replacements.iter() {
    //             let re = RegexBuilder::new(&replacement.search)
    //               .multi_line(true)
    //               .build()?;
    //             let replace_str = &replacement.replace;
    //             let mut found = false;
    //             let mut bad_replace = false;

    //             content = re
    //               .replace_all(&content, |caps: &Captures| {
    //                 found = true;

    //                 if let Some(m) = caps.name("begin") {
    //                   context
    //                     .set_value("begin".to_owned(), Value::from(m.as_str()))
    //                     .unwrap();
    //                 }
    //                 if let Some(m) = caps.name("end") {
    //                   context
    //                     .set_value("end".to_owned(), Value::from(m.as_str()))
    //                     .unwrap();
    //                 }
    //                 let result = eval_string_with_context(replace_str, context);

    //                 match result {
    //                   Ok(s) => s,
    //                   Err(_) => {
    //                     bad_replace = true;
    //                     String::new()
    //                   }
    //                 }
    //               })
    //               .into_owned();

    //             if bad_replace {
    //               return Err(From::from(format!(
    //                 "Replacement string '{}' generated an error",
    //                 replace_str
    //               )));
    //             }

    //             if !found {
    //               eprintln!(
    //                 "{}",
    //                 format!(
    //                   "'{}' Search/replace on '{}' did not match anything; check your search string '{}'",
    //                   "warning:",
    //                   target_file.display().to_string(),
    //                   replacement.search
    //                 )
    //                 .yellow()
    //               )
    //             }
    //           }

    //           if update {
    //             std::fs::write(&target_file, content)?;
    //             action += "Updated";
    //           } else {
    //             action += "Would update";
    //           }

    //           ()
    //         }
    //         Action::CopyFrom(from_expr) => {
    //           if update {
    //             let s = eval_string_with_context(from_expr, context)?;
    //             let from_file = version_file_dir.join(s);

    //             std::fs::copy(&from_file, &target_file).map_err(|_| {
    //               format!(
    //                 "unable to copy {} to {}",
    //                 from_file.display().to_string().bright_blue(),
    //                 target_file.display().to_string().bright_blue(),
    //               )
    //             })?;
    //             action += "Copied";
    //           } else {
    //             action += "Would copy"
    //           }
    //           ()
    //         }
    //         Action::Write(file_content) => {
    //           if update {
    //             std::fs::write(
    //               &target_file,
    //               eval_string_with_context(&file_content, context)?,
    //             )?;
    //             action += "Wrote";
    //           } else {
    //             action += "Would write";
    //           }
    //           ()
    //         }
    //       };

    //       eprintln!(
    //         "{} {} {}",
    //         action.bold().green(),
    //         target.description,
    //         target_file.display().to_string().bright_blue()
    //       );
    //     }
    //   }
    Ok(())
  }

  pub fn update_script_file(run_context: &HashMapContext) -> Result<(), Box<dyn Error>> {
    //   let mut new_content = content;

    //   let vars: HashMap<String, VarType>;

    //   for (identifier, _) in vars.iter() {
    //     if let Some(value) = context.get_value(identifier) {
    //       let s = match value {
    //         Value::String(s) => format!("\"{}\"", s),
    //         Value::Float(f) => format!("{}", f),
    //         Value::Boolean(b) => format!("{}", b),
    //         Value::Int(n) => format!("{}", n),
    //         _ => "".to_string(),
    //       };
    //       let re = RegexBuilder::new(
    //         &("(?P<begin>vars:\\s*\\{\n(?:.*\n)*?\\s*".to_string()
    //           + &identifier
    //           + "\\s*:\\s).*?(?P<end>\\s*,.*?\n)"),
    //       )
    //       .multi_line(true)
    //       .build()?;

    //       new_content = re
    //         .replace(&new_content, "${begin}".to_string() + &s + "${end}")
    //         .into_owned();
    //     }
    //   }

    // TODO: Write out the new script file content
    Ok(())
  }

  // #[cfg(test)]
  // mod tests {
  //   use super::*;

  //   #[test]
  //   fn test_run() {
  //     let temp_dir = tempfile::tempdir().unwrap();
  //     let version_file = temp_dir.path().join("version.json5");
  //     let version_content = r##"
  // {
  //   vars: {
  //     major: 3,
  //     minor: 0,
  //     patch: 0,
  //     build: 20210902,
  //     revision: 0,
  //     tz: "America/Los_Angeles",
  //     sequence: 6,
  //     buildType: "test",
  //     pi: 3.14,
  //     debug: true,
  //   },
  //   calcVars: {
  //     nextBuild: "now::year * 10000 + now::month * 100 + now::day",
  //     nextSequence: "sequence + 1",
  //   },
  //   operations: {
  //     incrMajor: "major += 1; minor = 0; patch = 0; revision = 0; build = nextBuild",
  //     incrMinor: "minor += 1; patch = 0; revision = 0; build = nextBuild",
  //     incrPatch: "patch += 1; revision = 0; build = nextBuild",
  //     incrRevision: "revision += 1; build = nextBuild",
  //     incrSequence: "sequence += 1",
  //     setBetaBuild: 'buildType = "beta"',
  //     setProdBuild: 'buildType = "prod"',
  //   },
  //   targets: [
  //     {
  //       description: "NodeJS package file",
  //       files: ["package.json"],
  //       action: {
  //         updates: [
  //           {
  //             search: '^(?P<begin>\\s*"version"\\s*:\\s*")\\d+\\.\\d+\\.\\d+(?P<end>")',
  //             replace: 'begin + str::from(major) + "." + str::from(minor) + "." + str::from(patch) + end',
  //           },
  //         ],
  //       },
  //     },
  //     {
  //       description: "TypeScript version",
  //       files: ["version.ts"],
  //       action: {
  //         updates: [
  //           {
  //             search: '^(?P<begin>\\s*export\\s*const\\s*version\\s*=\\s*")\\d+\\.\\d+\\.\\d+(?P<end>";?)$',
  //             replace: 'begin + str::from(major) + "." + str::from(minor) + "." + str::from(patch) + end',
  //           },
  //           {
  //             search: '^(?P<begin>\\s*export\\s*const\\s*fullVersion\\s*=\\s*")\\d+\\.\\d+\\.\\d+\\+\\d+\\.\\d+(?P<end>";?)$',
  //             replace: 'begin + str::from(major) + "." + str::from(minor) + "." + str::from(patch) + "+" + str::from(build) + "." + str::from(revision) + end',
  //           },
  //         ],
  //       },
  //     },
  //     {
  //       description: "Git version tag commit message",
  //       files: ["version.desc.txt"],
  //       action: {
  //         write: '"Version" + str::from(major) + "." + str::from(minor) + "." + str::from(patch) + "+" + str::from(build) + "." + str::from(revision)',
  //       },
  //     },
  //     {
  //       description: "Google Firebase",
  //       files: ["some-file.plist"],
  //       action: {
  //         copyFrom: '"some-file" + if(buildType == "test", "-test", "-prod") + ".plist"',
  //       },
  //     },
  //   ],
  // }
  //         "##;
  //     let package_json_content = r##"
  // {
  //   "name": "dummy",
  //   "version": "0.0.0"
  // }
  //     "##;
  //     let version_ts_content = r##"
  // export const version = "10.0.0";
  // export const fullVersion = "10.0.0+20210903.0";
  //     "##;

  //     std::fs::write(&version_file, &version_content).unwrap();
  //     std::fs::write(&temp_dir.path().join("package.json"), &package_json_content).unwrap();
  //     std::fs::write(&temp_dir.path().join("version.ts"), &version_ts_content).unwrap();
  //     std::fs::write(
  //       &temp_dir.path().join("some-file-test.plist"),
  //       "some-file-test",
  //     )
  //     .unwrap();

  //     run("incrMajor", Some(version_file.to_str().unwrap()), false).unwrap();
  //     run("incrMajor", Some(version_file.to_str().unwrap()), true).unwrap();
  //   }
  // }
}
