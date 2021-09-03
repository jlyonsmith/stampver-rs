# Version Stamping Tool (Rust Edition)

[![coverage](https://shields.io/endpoint?url=https://raw.githubusercontent.com/jlyonsmith/stampver-rs/main/coverage.json)](https://github.com/jlyonsmith/stampver-rs/blob/main/coverage.json)
[![Crates.io](https://img.shields.io/crates/v/stampver-rs.svg)](https://crates.io/crates/stampver-rs)
[![Docs.rs](https://docs.rs/stampver-rs/badge.svg)](https://docs.rs/stampver-rs)

A Rust package and command line tool for updating version information in ANY type of project.

- Define which files need to be updated
- Three types of actions; update in place, create or copy in existing files
- Use regular expressions to find and replace content in existing files
- Store and insert other information in addition to versions, such as copyrights, etc..
- Fully customize the type of version update operations that you want for your project
- Supports any type of versioning scheme that you wish to use

## Overview

Releasing a new project typically involves:

- Updating version numbers in `package.json`, `Cargo.toml`, plus any source code files
- Creating Git tags with version numbers
- Building and ensuring all tests pass
- Pushing changes to a cloud source repository, e.g. GitHub, GitLab, etc..
- Publishing the newly minted and versioned package to a cloud package repository.

All of the above steps can be simplified with the use of this tool.

To use the tool for your project, simply:

1. Place a `version.json5` file in your project root that:
     - Describes the files that hold version numbers in your project
     - Which of the three actions (update, write or copy-in) to perform on each file
     - The types of version update operations you want to perform (`incrMajor`, `incrMinor`, etc..)
2. Run `stampver` as part of your project's release script

Once you start using `stampver` you will be able to copy the `version.json5` file in from another project and tweak it slightly in order to get set up quickly.

## Command Line

The command line tool `stampver` is included in this crate using the `cli` feature flag, which is installed by default.

```text
$ stampver --help

StampVer 1.0.0+20210829.1
John Lyon-Smith
Version Update Tool.

USAGE:
    stampver [FLAGS] [OPTIONS] <OPERATION>

FLAGS:
    -h, --help       Prints help information
    -u, --update     Actually do the update
    -V, --version    Prints version information

OPTIONS:
    -i, --input <input_file>    Specify the version file explicitly

ARGS:
    <OPERATION>    Select update operation specified in the version file
```

The tool will describe the actions that it is taking on each file so you can check that it is doing what you expect.

## Expressions

This package uses the [evalexpr](https://crates.io/crates/evalexpr) to provide the ability to customize the different calculations and operations. `stampver` adds the following variables and functions:

| Variable   | Type | Purpose                      |
| ---------- | ---- | ---------------------------- |
| now::year  | Int  | Current UTC year             |
| now::month | Int  | Current UTC month            |
| now::day   | Int  | Current UTC day of the month |

And the following functions:

| Function    | Purpose                                                                  |
| ----------- | ------------------------------------------------------------------------ |
| if(a, b, c) | If expression `a` is `true` then the value of `b`, else the value of `c` |

`stampver` uses the [Regex](https://crates.io/crates/regex) crate for regular expressions. You can use the amazing [Regex101](https://regex101.com/) site to develop and test your own regular expressions.  Use the PCRE2 flavor of regular expressions for the most compatability with the `Regex` crate.

## Schema File Format

Here's an annotated schema file format:

```json5
{
  vars: {
    major: 3,
    minor: 0,
    patch: 0,
    build: 20210902,
    revision: 0,
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
      description: "JavaScript Files",
      files: ["src/version.js"],
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
      description: "Git Version Tag",
      files: ["scratch/version.tag.txt"],
      action: {
        write: 'str::from(major) + "." + str::from(minor) + "." + str::from(patch)',
      },
    },
    {
      description: "iOS PList",
      files: ["some-file.plist"],
      action: {
        copyFrom: '"src/some-file" + if(buildType == "test", "-test", "-prod") + ".plist"',
      },
    },
  ],
}
```

Because the format is [JSON5](https://json5.org/) and a superset of JSON you can freely use comments. It is recommended to use [Prettier](https://prettier.io/) or equivalent. This is not just to keep your file nicely formatted, but also because `stampver` needs to update the file it might get confused if the formatting is too different from the above.

The 4 main sections are as follows.

### `vars`

This is where the version information lives, so in effect it is a simple version information database for you project.  This is also the only section that the tool rewrites when version information is updated.  It does it such a way as to preserve comments, but the tool does expect the layout to be like the example above.

### `calcVars`

These are any variables that need to get generated each time the tool runs. This can include things like a `build` number that is based on the date, or a `nextSequence` number.  The values in this section are merged with the `vars`, so be wary of naming conflicts.

### `operations`

These are the different version operations for your project. `incrMajor`, `incrMinor`, `incrPatch` are typical operations, but you can add whatever makes sense.

### `targets`

`targets` is a array of objects containing a `description`, an array of `files` to update and then an `action`.  `action` contains exactly one of

- `updates` - An array of `{ search: , replace: }` objects.  `search` is a regular expression. It can contain at most two optional capture groups that must be called `begin` and `end`.  These can be used in the `replace` substitution string.
- `write` - Writes content to the target files.  The content is an expression.
- `copyFrom` - Copies a file from another file, relative to the location of the `version.json5` file.  The name of the other file is an expression.

## License

This package is distributed under the terms of the [Unlicense](http://unlicense.org/) license. See the [`UNLICENSE`](UNLICENSE) file for details.
