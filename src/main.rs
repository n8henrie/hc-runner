use std::env;
use std::io::{self, Write};
use std::process::exit;

use std::{error, result};

type Error = Box<dyn error::Error + Send + Sync>;
type Result<T> = result::Result<T, Error>;

fn main() -> Result<()> {
    // Set URL at compile time for release builds but not for debug builds to
    // facilitate testing with mock server
    let url = if cfg!(debug_assertions) {
        env::var("URL")?
    } else {
        env!("URL").to_string()
    };
    let args = env::args();
    let exit_code = runner::run(url, args)?;
    io::stdout().flush()?;
    io::stderr().flush()?;
    exit(exit_code);
}
