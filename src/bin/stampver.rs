use clap::{App, Arg};
use colored::Colorize;
use stampver::*;
use std::error::Error;
use std::path::Path;
use walkdir::WalkDir;

// {grcov-excl-start}
fn main() {
  let matches = App::new("StampVer")
    .version("0.1.0+20210904.3")
    .author("John Lyon-Smith")
    .about("Version Update Tool.")
    .arg(
      Arg::with_name("operation")
        .help("Select update operation specified in the version file")
        .value_name("OPERATION")
        .index(1)
        .required(true),
    )
    .arg(
      Arg::with_name("input_file")
        .help("Specify the version file explicitly")
        .long("input")
        .short("i")
        .takes_value(true),
    )
    .arg(
      Arg::with_name("update")
        .help("Actually do the update")
        .long("update")
        .short("u")
        .takes_value(false),
    )
    .get_matches();

  let result = run(
    matches.value_of("operation").unwrap(),
    matches.value_of("input_file"),
    matches.is_present("update"),
  );

  if let Err(ref err) = result {
    eprintln!("{} {}", "error:".red(), err.to_string().red());
    std::process::exit(-1);
  }
}
// {grcov-excl-end}

pub fn run(operation: &str, input_file: Option<&str>, update: bool) -> Result<(), Box<dyn Error>> {
  let version_file = match input_file {
    Some(input_file) => Path::new(input_file).canonicalize()?,
    None => {
      let mut path = None;

      for entry in WalkDir::new(".").contents_first(true) {
        if let Ok(entry) = entry {
          if entry.file_type().is_file()
            && entry
              .file_name()
              .to_str()
              .map(|s| s.starts_with("version.json"))
              .unwrap_or(false)
          {
            path = Some(Path::new(entry.path()).to_path_buf());
            break;
          }
        }
      }
      if path.is_none() {
        return Err(From::from(format!("No version.json file found")));
      }

      path.unwrap()
    }
  };

  let mut content = std::fs::read_to_string(&version_file)?;
  let version_info = json5::from_str::<VersionInfo>(&content)?;
  let mut context = create_run_context(&version_info)?;

  run_operation(operation, &version_info, &mut context)?;

  process_targets(
    &version_file.parent().unwrap(),
    &version_info,
    update,
    &mut context,
  )?;

  content = update_version_content(content, &version_info.vars, &context)?;

  if update {
    std::fs::write(&version_file, content)?;
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_run() {
    let temp_dir = tempfile::tempdir().unwrap();
    let version_file = temp_dir.path().join("version.json5");
    let version_content = r##"
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
      files: ["version.ts"],
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
      files: ["version.desc.txt"],
      action: {
        write: '"Version" + str::from(major) + "." + str::from(minor) + "." + str::from(patch) + "+" + str::from(build) + "." + str::from(revision)',
      },
    },
    {
      description: "Google Firebase",
      files: ["some-file.plist"],
      action: {
        copyFrom: '"some-file" + if(buildType == "test", "-test", "-prod") + ".plist"',
      },
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

    std::fs::write(&version_file, &version_content).unwrap();
    std::fs::write(&temp_dir.path().join("package.json"), &package_json_content).unwrap();
    std::fs::write(&temp_dir.path().join("version.ts"), &version_ts_content).unwrap();
    std::fs::write(
      &temp_dir.path().join("some-file-test.plist"),
      "some-file-test",
    )
    .unwrap();

    run("incrMajor", Some(version_file.to_str().unwrap()), false).unwrap();
    run("incrMajor", Some(version_file.to_str().unwrap()), true).unwrap();
  }
}
