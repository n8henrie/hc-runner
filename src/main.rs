use std::env;
use std::io::{self, Write};
use std::process::exit;

use std::{error, result};

type Error = Box<dyn error::Error + Send + Sync>;
type Result<T> = result::Result<T, Error>;

fn main() -> Result<()> {
    // Use compiled-in URL by default but fall back to runtime for testing
    let url = option_env!("URL")
        .map(ToOwned::to_owned)
        .unwrap_or(env::var("URL")?);

    let args = env::args();
    let exit_code = runner::run(url, args)?;
    io::stdout().flush()?;
    io::stderr().flush()?;
    exit(exit_code);
}
