#![warn(clippy::pedantic)]

use std::io::{self, Write};
use std::process::ExitCode;

use tracing_subscriber::{self, EnvFilter};

use hc_runner::{run, Config, Error, Result};

#[tokio::main]
async fn main() -> Result<ExitCode> {
    let config = Config::resolve().map_err(|err| {
        if let Error::Cli(e) = err {
            e.exit();
        } else {
            err
        }
    })?;

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("hyper=warn".parse()?)
                .add_directive("reqwest=warn".parse()?),
        )
        .with_max_level(config.verbosity)
        .init();

    let exit_code = run(config).await?;
    io::stdout().flush()?;
    io::stderr().flush()?;
    Ok(ExitCode::from(exit_code))
}
