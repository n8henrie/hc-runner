#![warn(clippy::pedantic)]

use std::env;
use std::io::{self, Write};
use std::process::{exit, Command};
use std::time::Duration;
use std::{error, result};

use serde::Serialize;
use serde_json::json;

type Error = Box<dyn error::Error + Send + Sync>;
type Result<T> = result::Result<T, Error>;

#[derive(Debug, Serialize)]
struct RunnerMessage {
    name: String,
    message: String,
    exitcode: u8,
}

fn main() -> Result<()> {
    let url = env!("URL");
    let mut args = env::args().into_iter().skip(1);
    let name = args
        .next()
        .ok_or_else(|| Error::from("no script name provided"))?;

    let output = Command::new("/usr/bin/caffeinate").args(args).output()?;

    let status = output.status;
    let (stdout, stderr) = (output.stdout, output.stderr);

    io::stdout().write_all(&stdout)?;
    io::stderr().write_all(&stderr)?;

    let exit_code = if status.success() {
        0
    } else {
        status.code().expect("could not determine status code")
    };

    let data = json!({
        "name":     name,
        "message":  stderr,
        "exit_code": exit_code,
    });

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let res = client.post(url).body(data.to_string()).send()?;

    if !res.status().is_success() {
        let text = res.text()?;
        println!("failed to update status: {text}");
    }

    io::stdout().flush()?;
    io::stderr().flush()?;
    exit(exit_code);
}
