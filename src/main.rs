use std::path::PathBuf;

use advanzia2csv::advanzia2csv;
use anyhow::Result;
use clap::{command, Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to PDF file or folder that contains PDF files
    input: PathBuf,
    /// Path to output CSV file
    output: PathBuf,
    /// Swap sign of the amount
    #[arg(long, default_value_t = false)]
    swap_sign: bool,
    /// Log level
    #[arg(short = 'l', long = "log-level", value_enum, default_value = "info")]
    log_level: LogLevel,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

fn init_logger(level: LogLevel) {
    let level_str = format!("{:?}", level).to_lowercase();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&level_str))
        .target(env_logger::Target::Stderr)
        .init();
}

fn main() -> Result<()> {
    let args = Args::parse();
    init_logger(args.log_level);
    advanzia2csv(&args.input, &args.output, args.swap_sign)
}
