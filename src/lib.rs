use std::env::Args;
use std::io::{self, Write};
use std::process::Command;
use std::time::Duration;
use std::{error, result};

use serde::{Deserialize, Serialize};
use serde_json::json;

type Error = Box<dyn error::Error + Send + Sync>;
type Result<T> = result::Result<T, Error>;

#[derive(Debug, Deserialize, Serialize)]
pub struct RunnerMessage {
    pub name: String,
    pub message: String,
    pub exit_code: u8,
}

pub fn run(url: impl AsRef<str>, args: Args) -> Result<i32> {
    let mut args = args.into_iter().skip(1);
    let name = args
        .next()
        .ok_or_else(|| Error::from("no script name provided"))?;

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
        println!("failed to update status: {text}");
    }
    Ok(exit_code)
}
