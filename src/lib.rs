#![warn(clippy::pedantic)]

use std::fmt;
use std::io::{self, Write};
use std::process::Command;
use std::time::Duration;

use reqwest::{Client, Url};
use tracing::{info, warn};

extern crate config as config_rs;

pub type Result<T> = std::result::Result<T, Error>;

mod config;
pub use config::Config;

#[derive(thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Cli(#[from] clap::error::Error),

    /// Logical error in configuration
    #[error("config error: {0}")]
    Config(String),

    #[error(transparent)]
    EnvVar(#[from] std::env::VarError),

    /// Error with configuration file or environment variables
    #[error("settings error: {0}")]
    Settings(#[from] config_rs::ConfigError),

    #[error("command exited with empty exit status code")]
    EmptyExitCode,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),

    #[error(transparent)]
    ParseFilter(#[from] tracing_subscriber::filter::ParseError),

    #[error(transparent)]
    ParseUrl(#[from] url::ParseError),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    TryFromInt(#[from] std::num::TryFromIntError),

    #[error("unknown hc-runner error")]
    Unknown,

    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

fn add_slug(mut url: Url, slug: String) -> Result<Url> {
    // Calls to `join` will only interpret the last segment of the path as a
    // directory if it has a trailing slash
    // https://docs.rs/reqwest/latest/reqwest/struct.Url.html#method.join
    let path = url.path();
    if !path.ends_with('/') {
        url.set_path(&(path.to_string() + "/"));
    }

    let with_slug = url.join(&(slug + "/"))?;
    Ok(with_slug)
}

/// # Errors
/// Returns the exit code of the command
#[tracing::instrument]
pub async fn run(config: Config) -> Result<u8> {
    let url = add_slug(config.url, config.slug)?;
    info!("using base url: {}", url);

    let client = Client::builder()
        .timeout(Duration::from_secs(config.timeout))
        .build()?;

    // Some commands can be allowed to fail periodically and I only want a
    // healthchecks notification if there are zero successes in a period of
    // time. For these, use the `--success-only` flag, which will only update
    // healthchecks when there is a successful run.
    let start_req = if config.success_only {
        None
    } else {
        let client = client.clone();
        let mut url = url.join("start")?;
        Some(tokio::spawn(async move {
            url.set_query(Some("create=1"));
            info!("calling start url {}", url);
            client.head(url).send().await
        }))
    };

    let output = if cfg!(target_os = "macos") {
        Command::new("/usr/bin/caffeinate")
            .args(config.command)
            .output()?
    } else {
        let mut args = config.command.iter();
        let cmd = args
            .next()
            .ok_or_else(|| Error::Config("command was empty".into()))?;
        Command::new(cmd).args(args).output()?
    };

    let (stdout, stderr) = (output.stdout, output.stderr);
    io::stdout().write_all(&stdout)?;
    io::stderr().write_all(&stderr)?;

    let status = output.status;
    let exit_code = if status.success() {
        0
    } else {
        status.code().ok_or_else(|| Error::EmptyExitCode)?
    };

    let stderr = std::str::from_utf8(&stderr)?;

    if let Some(req) = start_req {
        let _ = req.await?;
    }

    match (config.success_only, exit_code) {
        (false, _) | (true, 0) => {
            let res = {
                let url = url.join(exit_code.to_string().as_ref())?;
                info!("calling end url {}", url);
                client.post(url).body(stderr.to_string()).send().await?
            };

            if !res.status().is_success() {
                let text = res.text().await?;
                writeln!(io::stderr(), "failed to update status: {text}")?;
            }
        }
        _ => (),
    }

    Ok(exit_code.try_into()?)
}
