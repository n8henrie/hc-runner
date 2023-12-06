#![warn(clippy::pedantic)]
use std::env::Args;
use std::io::{self, Write};
use std::process::{exit, Command};
use std::time::Duration;
use std::{error, result};

use serde::{Deserialize, Serialize};
use serde_json::json;

use chrono::prelude::*;

type Error = Box<dyn error::Error + Send + Sync>;
type Result<T> = result::Result<T, Error>;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Debug, Deserialize, Serialize)]
pub struct RunnerMessage {
    pub name: String,
    pub message: String,
    pub exit_code: u8,
}

/// # Errors
/// Returns the exit code of the command
pub fn run(url: impl AsRef<str>, args: &mut Args) -> Result<i32> {
    // Discard $0
    let _ = args.next();

    let name = args.next();
    let name = match name.as_deref() {
        Some("-V" | "--version") => {
            writeln!(io::stdout(), "{CARGO_PKG_NAME} version {VERSION}")?;
            exit(0);
        }
        Some(name) => name,
        None => return Err(Error::from("no script name provided")),
    };

    let now = Local::now();
    writeln!(io::stderr(), "Starting `{name}` at {now}")?;

    let output = if cfg!(target_os = "macos") {
        Command::new("/usr/bin/caffeinate").args(args).output()?
    } else {
        let cmd = args
            .next()
            .ok_or_else(|| Error::from("No command specified"))?;
        Command::new(cmd).args(args).output()?
    };

    let (stdout, stderr) = (output.stdout, output.stderr);
    io::stdout().write_all(&stdout)?;
    io::stderr().write_all(&stderr)?;

    let status = output.status;
    let exit_code = if status.success() {
        0
    } else {
        status
            .code()
            .ok_or_else(|| Error::from("could not determine status code"))?
    };

    let stderr = std::str::from_utf8(&stderr)?;
    let data = json!({
        "name":     name,
        "message":  stderr,
        "exit_code": exit_code,
    });

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let res = client.post(url.as_ref()).body(data.to_string()).send()?;

    if !res.status().is_success() {
        let text = res.text()?;
        writeln!(io::stderr(), "failed to update status: {text}")?;
    }

    let now = Local::now();
    writeln!(io::stderr(), "Ending `{name}` at {now}")?;
    Ok(exit_code)
}
