#![warn(clippy::pedantic)]
use std::env;
use std::io::{self, Write};
use std::process::exit;

use tracing::warn;
use tracing_subscriber::{self, EnvFilter};

use runner::{run, Result};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("hyper=warn".parse()?)
                .add_directive("reqwest=warn".parse()?),
        )
        .init();

    #[cfg(feature = "mocks")]
    let url: &str =
        &env::var("URL").expect("Missing environment variable: URL");

    #[cfg(not(feature = "mocks"))]
    let url: &str = env!("URL");

    let mut args = env::args();
    let exit_code = run(url, &mut args).await?;
    io::stdout().flush()?;
    io::stderr().flush()?;
    exit(exit_code);
}
