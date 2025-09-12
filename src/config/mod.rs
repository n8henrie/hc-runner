use std::{
    io::{self, Write},
    path::PathBuf,
};

use crate::{Error, Result};
use clap::builder::NonEmptyStringValueParser;
use clap::Parser;
use reqwest::Url;
use tracing::Level;

use directories::ProjectDirs;

extern crate config as config_rs;
use config_rs::{Environment, File};
use serde::Deserialize;

#[derive(Clone, Debug, Parser)]
#[command(author, version, about, long_about)]
struct Cli {
    #[arg(trailing_var_arg(true), required(true), value_parser=NonEmptyStringValueParser::new())]
    pub(crate) command: Vec<String>,

    /// Specify a config file in non-default location
    #[arg(short, long)]
    pub(crate) config: Option<PathBuf>,

    /// Silence logging / warnings. Does not affect called command's output.
    #[arg(short, long, conflicts_with("verbose"))]
    pub quiet: bool,

    /// Set healthchecks slug for this call.
    #[arg(short, long, value_name = "NAME", value_parser=NonEmptyStringValueParser::new())]
    pub(crate) slug: String,

    /// Disable calling `/start` and only ping healthchecks if the test was successful.
    #[arg(long)]
    pub(crate) success_only: bool,

    /// Set timeout for requests to healthchecks server.
    #[arg(short, long)]
    pub(crate) timeout: Option<u64>,

    /// Specify the URL of the healthchecks server for this call.
    #[arg(short, long)]
    pub(crate) url: Option<Url>,

    /// Increase logging verbosity. May be repeated. Defaults to `Level::WARN`.
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

/// Settings that are configurable via config file or environment variables
/// Order of priority (higher numbers override lower)
/// 1. Config file
/// 2. Environment variables
/// 3. CLI flags
#[derive(Debug, Deserialize)]
struct Settings {
    url: Option<Url>,
    timeout: Option<u64>,
}

fn parse_verbosity(n: u8) -> Level {
    match n.saturating_add(1) {
        0 => Level::ERROR,
        1 => Level::WARN,
        2 => Level::INFO,
        3 => Level::DEBUG,
        _ => Level::TRACE,
    }
}

#[derive(Debug)]
pub struct Config {
    pub(crate) command: Vec<String>,
    pub(crate) slug: String,
    pub(crate) success_only: bool,
    pub(crate) timeout: u64,
    pub(crate) url: Url,
    pub verbosity: Level,
}

impl Config {
    #[tracing::instrument]
    pub fn resolve() -> Result<Self> {
        let cli = Cli::try_parse()?;
        Self::resolve_with(cli)
    }

    fn resolve_with(cli: Cli) -> Result<Self> {
        let mut builder = config_rs::Config::builder();

        let conf_file = cli.config.or_else(|| {
            ProjectDirs::from("com", "n8henrie", "hc-runner")
                .map(|pd| pd.config_dir().join("config.toml"))
        });

        if let Some(conf_file) = conf_file {
            // tracing not configured until after this method returns, so
            // this is a non-pretty workaround to help users find where the
            // config file should be placed
            if cli.verbose >= 2 {
                writeln!(
                    io::stderr(),
                    "searching for config file at {}",
                    conf_file.display(),
                )?;
            }
            builder =
                builder.add_source(File::from(conf_file).required(false));
        }
        let settings: Settings = builder
            .add_source(Environment::with_prefix("HC_RUNNER"))
            .build()?
            .try_deserialize()?;

        let url = cli
            .url
            .or(settings.url)
            .ok_or_else(|| Error::Config("Base URL not found".into()))?;

        let timeout: u64 = cli.timeout.or(settings.timeout).unwrap_or(10);

        let verbosity =
            parse_verbosity(if cli.quiet { 0 } else { cli.verbose });
        let Cli {
            command,
            slug,
            success_only,
            ..
        } = cli;

        Ok(Self {
            command,
            slug,
            success_only,
            timeout,
            url,
            verbosity,
        })
    }
}

#[cfg(test)]
mod tests;
