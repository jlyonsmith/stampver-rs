use clap::{App, Arg};
use stampver::*;
use std::error::Error;
use std::fs::File;
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

pub fn run(operation: &str, input_file: Option<&str>, update: bool) -> Result<(), Box<dyn Error>> {
    let version_file = match input_file {
        Some(input_file) => Path::new(input_file).canonicalize()?,
        _ => panic!("Need to search for version file"),
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
    } else {
        println!("{}", content)
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_tabs() {}
}
