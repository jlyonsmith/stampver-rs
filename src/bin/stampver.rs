use clap::{arg_enum, value_t, App, Arg};
use stampver::*;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

// {grcov-excl-start}
fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("StampVer")
        .version("1.0.0+20210829.1")
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
        eprintln!("error: {}", err);
    }

    result
}
// {grcov-excl-end}

pub fn run(
    operation: &str,
    input_file: Option<&str>,
    round_down: bool,
) -> Result<(), Box<dyn Error>> {
    if let Some(input_file) = input_file {
        read_version_file(&mut File::open(Path::new(input_file))?)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_tabs() {}
}
