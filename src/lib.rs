//! Update version information in project files

use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::read_to_string;
use std::io::{Read, Write};

#[derive(Deserialize, PartialEq, Debug)]
pub struct UpdateAction {
  search: String,
  replace: String,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum VarType {
  Bool(bool),
  String(String),
  Number(f64),
}

#[derive(Deserialize, PartialEq, Debug)]
pub enum Action {
  #[serde(rename = "updates")]
  Update(Vec<UpdateAction>),
  #[serde(rename = "write")]
  Write(String),
  #[serde(rename = "copyFrom")]
  CopyFrom(String),
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct VersionTarget {
  description: String,
  files: Vec<String>,
  action: Action,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct VersionInfo {
  vars: HashMap<String, VarType>,
  #[serde(rename = "calcVars")]
  calc_vars: HashMap<String, String>,
  operations: HashMap<String, String>,
  targets: Vec<VersionTarget>,
}

pub fn read_version_file(reader: &mut dyn Read) -> Result<(), Box<dyn Error>> {
  let mut content = String::new();

  reader.read_to_string(&mut content)?;

  json5::from_str::<VersionInfo>(&content)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_read_version_file() {
    let mut input = r##"
{
  vars: {
    major: 1,
    debug: true,
    tz: "America/Los_Angeles",
  },
  calcVars: {
    nextBuild: "year * 10000",
  },
  operations: {
    incrMajor: "major += 1",
  },
  targets: [
    {
      description: "TypeScript version",
      files: ["src/version.ts"],
      action: {
        updates: [
          {
            search: '^(?<begin>\\s*export\\s*const\\s*version\\s*=\\s*")\\d+\\.\\d+\\.\\d+(?<end>"\\s*)$',
            replace: "${begin}${major}.${minor}.${patch}${end}",
          },
          {
            search: '^(?<begin>\\s*export\\s*const\\s*fullVersion\\s*=\\s*")\\d+\\.\\d+\\.\\d+\\+\\d+\\.\\d+(?<end>"\\s*)$',
            replace: "${begin}${major}.${minor}.${patch}+${build}.${revision}${end}",
          },
        ],
      },
    },
    {
      description: "Git version tag",
      files: ["scratch/version.tag.txt"],
      action: { write: "${major}.${minor}.${patch}" },
    },
    {
      description: "Google Firebase",
      files: ["src/some-file.plist"],
      action: {
        copyFrom: "('src/some-file${buildType' === 'test' ? '-test' : '-prod') + '.plist'",
      },
    },
  ],
}
    "##.as_bytes();

    assert_eq!((), read_version_file(&mut input).unwrap());
  }
}
