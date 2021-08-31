# Version Stamping Tool (Rust Edition)

[![coverage](https://shields.io/endpoint?url=https://raw.githubusercontent.com/jlyonsmith/stampver-rs/main/coverage.json)](https://github.com/jlyonsmith/stampver-rs/blob/main/coverage.json)
[![Crates.io](https://img.shields.io/crates/v/stampver-rs.svg)](https://crates.io/crates/stampver-rs)
[![Docs.rs](https://docs.rs/stampver-rs/badge.svg)](https://docs.rs/stampver-rs)

A Rust package and command line tool for updating version information in ANY type of project.

- Can define which files need to be updated
- Files can be update in place or created
- Use regular expressions to find and replace content in existing files
- Can store other information in addition to versions, such as copyrights, etc..
- Can fully customize the type of version update operations that you desire
- Can support any type of versioning scheme that you wish to use

## Command Line

The command line tool `stampver` is included in this crate using the `cli` feature flag, which is installed by default.

```sh
```

## License

This package is distributed under the terms of the [Unlicense](http://unlicense.org/) license. See the [`UNLICENSE`](UNLICENSE) file for details.
