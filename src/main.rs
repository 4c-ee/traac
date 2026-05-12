mod config;
mod lastfm;
mod mpris;
mod ui;

use std::process::ExitCode;

fn main() -> ExitCode {
    match ui::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}