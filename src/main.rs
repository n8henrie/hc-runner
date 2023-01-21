#![warn(clippy::pedantic)]
use std::env;
use std::io::{self, Write};
use std::process::exit;

use std::{error, result};

type Error = Box<dyn error::Error + Send + Sync>;
type Result<T> = result::Result<T, Error>;

const URL: &str = env!("URL");

fn main() -> Result<()> {
    let args = env::args();
    let exit_code = runner::run(URL, args)?;
    io::stdout().flush()?;
    io::stderr().flush()?;
    exit(exit_code);
}
