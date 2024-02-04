#![warn(clippy::pedantic)]

use std::io::{self, Write};
use std::process::ExitCode;

use tracing::{warn, Level};
use tracing_subscriber::{self, EnvFilter};

use hc_runner::{run, Config, Result};

use clap::Parser;

fn parse_verbosity(n: u8) -> Level {
    match n {
        0 => Level::ERROR,
        1 => Level::WARN,
        2 => Level::INFO,
        3 => Level::DEBUG,
        _ => Level::TRACE,
    }
}

#[tokio::main]
async fn main() -> Result<ExitCode> {
    let config = Config::parse();

    let verbosity =
        parse_verbosity(if config.quiet { 0 } else { config.verbose });
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("hyper=warn".parse()?)
                .add_directive("reqwest=warn".parse()?),
        )
        .with_max_level(verbosity)
        .init();

    let exit_code = run(config).await?;
    io::stdout().flush()?;
    io::stderr().flush()?;
    Ok(ExitCode::from(exit_code))
}
