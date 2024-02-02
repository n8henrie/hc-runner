#![warn(clippy::pedantic)]
use std::borrow::Cow;
use std::env::Args;
use std::fmt::Debug;
use std::io::{self, Write};
use std::process::{exit, Command};
use std::time::Duration;
use std::{error, result};

use reqwest::{Client, Url};
use tracing::{info, warn};

type Error = Box<dyn error::Error + Send + Sync>;
pub type Result<T> = result::Result<T, Error>;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");

/// # Errors
/// Returns the exit code of the command
#[tracing::instrument]
pub async fn run<T>(url: T, args: &mut Args) -> Result<i32>
where
    T: Debug + AsRef<str>,
{
    // Discard $0
    let _ = args.next();

    let name = args.next();
    let name = match name.as_deref() {
        Some("-V" | "--version") => {
            writeln!(io::stdout(), "{CARGO_PKG_NAME} version {VERSION}")?;
            exit(0);
        }
        Some(name) => name.to_string(),
        None => return Err(Error::from("no script name provided")),
    };

    // `join` will only interpret the last segment of the path as a directory if it has a trailing slash
    // https://docs.rs/reqwest/latest/reqwest/struct.Url.html#method.join
    let url = {
        let url = url.as_ref();
        let url = if url.ends_with('/') {
            Cow::Borrowed(url)
        } else {
            Cow::Owned(String::from(url) + "/")
        };
        Url::parse(url.as_ref())?.join((name + "/").as_ref())?
    };
    info!("using base url: {}", url);

    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;

    let start_req = {
        let client = client.clone();
        let mut url = url.join("start")?;
        tokio::spawn(async move {
            url.set_query(Some("create=1"));
            info!("calling start url {}", url);
            client.head(url).send().await
        })
    };

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

    let _ = start_req.await?;
    let res = {
        let url = url.join(exit_code.to_string().as_ref())?;
        info!("calling end url {}", url);
        client.post(url).body(stderr.to_string()).send().await?
    };

    if !res.status().is_success() {
        let text = res.text().await?;
        writeln!(io::stderr(), "failed to update status: {text}")?;
    }

    Ok(exit_code)
}
