//! Update version information in project files

use chrono::prelude::*;
use colored::Colorize;
use evalexpr::*;
use regex::{Captures, RegexBuilder};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

#[derive(Deserialize, PartialEq, Debug)]
pub struct Replacement {
  pub search: String,
  pub replace: String,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum VarType {
  Bool(bool),
  String(String),
  Number(f64),
}

impl VarType {
  fn as_expr_value(self: &Self) -> Value {
    match self {
      VarType::Bool(b) => Value::Boolean(*b),
      VarType::String(s) => Value::String((*s).to_owned()),
      VarType::Number(n) => {
        if n.fract() == 0_f64 {
          Value::Int(*n as i64)
        } else {
          Value::Float(*n)
        }
      }
    }
  }
}

#[derive(Deserialize, PartialEq, Debug)]
pub enum Action {
  #[serde(rename = "updates")]
  Update(Vec<Replacement>),
  #[serde(rename = "write")]
  Write(String),
  #[serde(rename = "copyFrom")]
  CopyFrom(String),
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct VersionTarget {
  pub description: String,
  pub files: Vec<String>,
  pub action: Action,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct VersionInfo {
  pub vars: HashMap<String, VarType>,
  #[serde(rename = "calcVars")]
  pub calc_vars: HashMap<String, String>,
  pub operations: HashMap<String, String>,
  pub targets: Vec<VersionTarget>,
}

pub fn create_run_context(version_info: &VersionInfo) -> Result<HashMapContext, Box<dyn Error>> {
  let mut context = HashMapContext::new();

  // Add all fixed vars into the context
  for (identifier, value) in version_info.vars.iter() {
    context.set_value(identifier.to_string(), value.as_expr_value())?;
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
  for (identifier, value) in version_info.calc_vars.iter() {
    context.set_value(
      identifier.to_string(),
      evalexpr::eval_with_context(&value, &context)?,
    )?;
  }

  Ok(context)
}

pub fn run_operation(
  operation: &str,
  version_info: &VersionInfo,
  context: &mut HashMapContext,
) -> Result<(), Box<dyn Error>> {
  if let Some(value) = version_info.operations.get(operation) {
    eprintln!("{} {}", "Operation".bold().green(), operation);
    eval_with_context_mut(&value, context)?;
  } else {
    eprintln!("No operation named {} was found", operation.bright_blue());
  }

  Ok(())
}

pub fn process_targets(
  version_file_dir: &Path,
  version_info: &VersionInfo,
  update: bool,
  context: &mut HashMapContext,
) -> Result<(), Box<dyn Error>> {
  for target in version_info.targets.iter() {
    for target_file in target.files.iter() {
      let target_file = version_file_dir.join(target_file);
      let mut action = "".to_string();

      match &target.action {
        Action::Update(replacements) => {
          let mut content = std::fs::read_to_string(&target_file)?;

          for replacement in replacements.iter() {
            let re = RegexBuilder::new(&replacement.search)
              .multi_line(true)
              .build()?;
            let replace_str = &replacement.replace;
            let mut found = false;

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
                eval_string_with_context(replace_str, context).unwrap()
              })
              .into_owned();

            if !found {
              eprint!(
                "Search/replace on '{}' did not match anything; check your search string '{}'",
                target_file.display(),
                replacement.search
              )
            }
          }

          if update {
            std::fs::write(&target_file, content)?;
            action += "Updated";
          } else {
            action += "Would update";
          }

          ()
        }
        Action::CopyFrom(from_expr) => {
          if update {
            let s = eval_string_with_context(from_expr, context)?;
            let from_file = version_file_dir.join(s);

            std::fs::copy(&from_file, &target_file)?;
            action += "Copied";
          } else {
            action += "Would copy"
          }
          ()
        }
        Action::Write(file_content) => {
          if update {
            std::fs::write(
              &target_file,
              eval_string_with_context(&file_content, context)?,
            )?;
            action += "Wrote";
          } else {
            action += "Would write";
          }
          ()
        }
      };

      eprintln!(
        "{} {} {}",
        action.bold().green(),
        target.description,
        target_file.display().to_string().bright_blue()
      );
    }
  }

  Ok(())
}

pub fn update_version_content(
  content: String,
  vars: &HashMap<String, VarType>,
  context: &HashMapContext,
) -> Result<String, Box<dyn Error>> {
  let mut new_content = content;

  for (identifier, _) in vars.iter() {
    if let Some(value) = context.get_value(identifier) {
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
      .build()?;

      new_content = re
        .replace(&new_content, "${begin}".to_string() + &s + "${end}")
        .into_owned();
    }
  }

  Ok(new_content)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_read_version_file() {
    let mut input = r##"
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
      action: {
        updates: [
          {
            search: '^(?P<begin>\\s*"version"\\s*:\\s*")\\d+\\.\\d+\\.\\d+(?P<end>")',
            replace: 'begin + str::from(major) + "." + str::from(minor) + "." + str::from(patch) + end',
          },
        ],
      },
    },
    {
      description: "TypeScript version",
      files: ["src/version.ts"],
      action: {
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
    },
    {
      description: "Git version tag commit message",
      files: ["scratch/version.desc.txt"],
      action: {
        write: '"Version" + str::from(major) + "." + str::from(minor) + "." + str::from(patch) + "+" + str::from(build) + "." + str::from(revision)',
      },
    },
    {
      description: "Google Firebase",
      files: ["some-file.plist"],
      action: {
        copyFrom: '"src/some-file" + if(buildType == "test", "-test", "-prod") + ".plist"',
      },
    },
  ],
}
"##;

    let version_info = json5::from_str::<VersionInfo>(&input);
  }
}
