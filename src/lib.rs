#![warn(clippy::pedantic)]

use std::io::{self, Write};
use std::process::Command;
use std::time::Duration;
use std::{env, error, result};

use reqwest::Client;
use tracing::{info, warn};

type Error = Box<dyn error::Error + Send + Sync>;
pub type Result<T> = result::Result<T, Error>;

mod config;
use config::parse_url;
pub use config::Config;

fn default_url() -> Result<reqwest::Url> {
    parse_url(env::var("HC_RUNNER_URL")?.as_ref())
}

/// # Errors
/// Returns the exit code of the command
#[tracing::instrument]
pub async fn run(config: Config) -> Result<u8> {
    let name = config.slug;

    let url = config
        .url
        .map_or_else(default_url, Ok)?
        .join((name + "/").as_ref())?;
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

    if let Some(req) = start_req {
        let _ = req.await?;
    };

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
            };
        }
        _ => (),
    }

    Ok(exit_code.try_into()?)
}
