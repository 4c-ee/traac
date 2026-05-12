mod config;
mod lastfm;
mod mpris;
mod ui;

use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();

    if args.debug {
        std::env::set_var("RUST_LOG", "debug");
    } else if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    log::info!("Starting traac v{}", env!("CARGO_PKG_VERSION"));

    match ui::run(args.config) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            log::error!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}