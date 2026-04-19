use anyhow::Context;
use clap::Parser;
use env_logger::{Builder, Target};
use log::{Level, LevelFilter};
use stampver::{ScriptError, StampVerTool};
use std::{io::Write, path::PathBuf, process::exit};

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The versioning operation to perform
    #[arg(value_name = "OPERATION")]
    operation: Option<String>,

    /// Specify the version file explicitly
    #[arg(
        value_name = "INPUT_FILE",
        short,
        long = "input",
        default_value = "version.json5"
    )]
    input_file: PathBuf,

    /// Actually do the update
    #[arg(short, long)]
    update: bool,

    /// Filter output to update only files under this directory
    #[arg(value_name = "DIR_PATH", short = 'f', long = "filter")]
    filter_path: Option<PathBuf>,
}

fn main() {
    match run() {
        Ok(code) => exit(code),
        Err(err) => {
            eprintln!("error: {}", err);
            let mut source = err.source();
            while let Some(e) = source {
                eprintln!("caused by: {}", e);
                source = e.source();
            }
            exit(1);
        }
    }
}

/// Run the stampver tool with the given arguments.
pub fn run() -> anyhow::Result<i32> {
    let cli = match Cli::try_parse() {
        Ok(m) => m,
        Err(err) => {
            eprintln!("{}", err);
            return Ok(0);
        }
    };

    let mut builder = Builder::new();

    builder
        .format(|buf, record| match record.level() {
            Level::Info => writeln!(buf, "{}", record.args()),
            _ => writeln!(buf, "{}: {}", record.level(), record.args()),
        })
        .filter(None, LevelFilter::Info)
        .parse_default_env()
        .target(Target::Stderr)
        .init();

    let tool = StampVerTool::new();
    let (content, root_node, script_file) = tool
        .read_script_file(cli.input_file)
        .context("failed to read script file")?;
    let filter_path = tool.validate_filter_path(cli.filter_path.as_deref())?;

    let inner_run = || {
        tool.validate_script_file(&root_node)?;

        let mut run_context = tool.create_run_context(&root_node)?;

        tool.run_operation(cli.operation, &root_node, &mut run_context)?;
        tool.process_targets(
            &script_file,
            &root_node,
            cli.update,
            &mut run_context,
            &filter_path,
        )?;
        tool.update_script_file(&script_file, content, &root_node, &run_context, cli.update)?;

        Ok::<_, ScriptError>(())
    };

    inner_run().map_err(|e| ScriptError::new(e.message, Some(script_file.clone()), e.location))?;

    Ok(0)
}
