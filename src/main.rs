#![warn(clippy::pedantic)]
use std::env;
use std::io::{self, Write};
use std::process::exit;
use std::{error, result};

type Error = Box<dyn error::Error + Send + Sync>;
type Result<T> = result::Result<T, Error>;

fn main() -> Result<()> {
    #[cfg(feature = "mocks")]
    let url: &str =
        &env::var("URL").expect("Missing environment variable: URL");

    #[cfg(not(feature = "mocks"))]
    let url: &str = env!("URL");

    let mut args = env::args();
    let exit_code = runner::run(url, &mut args)?;
    io::stdout().flush()?;
    io::stderr().flush()?;
    exit(exit_code);
}
