mod parser;
mod splitter;
mod outputs;
mod setter;

use std::process;
use colored::Colorize;
use parser::AppConfig;
use splitter::Splitter;
use crate::outputs::Monitor;


// Read app and output config and use it to run splitter.
fn run() -> Result<(), String> {
    // read config
    let app_config = AppConfig::new()
        .map_err(|err| err.to_string())?;

    // fetch monitors
    let monitors = Monitor::get_monitors()
        .map_err(|err| err.to_string())?;

    // create new splitter
    let splitter = Splitter::new(&app_config, &monitors);

    // perform split
    splitter.run()
        .map_err(|err| err.to_string())?;

    Ok(())
}


// Run app, output errors.
fn main() {
    if let Err(err) = run() {
        eprintln!("{}: {}", "rwpspread".red().bold(), err);
        process::exit(1);
    }
}
