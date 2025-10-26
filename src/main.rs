use std::{fmt, path::PathBuf};

use advanzia2csv::advanzia2csv;
use anyhow::Result;
use clap::{command, Parser, ValueEnum};
use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;

#[derive(ValueEnum, Clone, Debug)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}
impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LogLevel::Error => write!(f, "error"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Trace => write!(f, "trace"),
        }
    }
}

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
    #[arg(short, long, default_value_t = LogLevel::Info)]
    log_level: LogLevel,
}

fn setup_logger(log_level: LevelFilter) -> Result<()> {
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Green)
        .debug(Color::Cyan)
        .trace(Color::Magenta);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                colors.color(record.level()),
                message
            ))
        })
        .level(log::LevelFilter::Warn) // Set the default level
        .level_for(module_path!(), log_level) // Set the default level
        .chain(std::io::stdout())
        .apply()?;

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    setup_logger(args.log_level.into())?;
    advanzia2csv(&args.input, &args.output, args.swap_sign)
}
