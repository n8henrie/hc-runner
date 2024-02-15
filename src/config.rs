use std::io::{self, Write};

use crate::{Error, Result};
use clap::builder::NonEmptyStringValueParser;
use clap::Parser;
use reqwest::Url;
use tracing::Level;

use directories::ProjectDirs;

extern crate config as config_rs;
use config_rs::{Environment, File};
use serde::Deserialize;

#[derive(Debug, Parser)]
#[command(author, version, about, about, long_about)]
struct Cli {
    #[arg(trailing_var_arg(true), required(true), value_parser=NonEmptyStringValueParser::new())]
    pub(crate) command: Vec<String>,

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
    #[arg(short, long, default_value("10"))]
    pub(crate) timeout: u64,

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
}

fn parse_verbosity(n: u8) -> Level {
    match n {
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

        let mut builder = config_rs::Config::builder();
        if let Some(pd) = ProjectDirs::from("com", "n8henrie", "hc-runner") {
            if let Some(conf_file) =
                pd.config_dir().join("config.toml").to_str()
            {
                // tracing not configured until after this method returns, so
                // this is a non-pretty workaround to help users find where the
                // config file should be placed
                if cli.verbose >= 2 {
                    writeln!(
                        io::stdout(),
                        "searching for config file at {conf_file}"
                    )?;
                };
                builder = builder
                    .add_source(File::with_name(conf_file).required(false));
            }
        };
        let settings: Settings = builder
            .add_source(Environment::with_prefix("HC_RUNNER"))
            .build()?
            .try_deserialize()?;

        let url = cli
            .url
            .or(settings.url)
            .ok_or_else(|| Error::Config("Base URL not found".into()))?;

        let verbosity =
            parse_verbosity(if cli.quiet { 0 } else { cli.verbose });
        let Cli {
            command,
            slug,
            success_only,
            timeout,
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
mod tests {
    use super::*;

    #[test]
    fn test_config_parser() {
        let config = Cli::parse_from([
            "",
            "--slug=fake",
            "-vv",
            "--",
            "echo",
            "-vvv",
            "foo",
            "bar",
            "foo bar",
        ]);
        assert_eq!(config.slug, "fake");
        assert_eq!(config.verbose, 2);
        assert_eq!(
            config.command,
            vec!["echo", "-vvv", "foo", "bar", "foo bar"]
        );
        for config in vec![
            Cli::parse_from(["", "--slug=fake", "cat"]),
            Cli::parse_from(["", "--slug", "fake", "--", "cat"]),
            Cli::parse_from(["", "-s", "fake", "cat"]),
        ] {
            assert_eq!(config.slug, "fake");
            assert!(!config.success_only);
            assert_eq!(config.command, vec!["cat"]);
        }
    }

    #[test]
    fn test_command_required() {
        assert!(
            Cli::try_parse_from(["", "--slug=no_command_no_dashes"]).is_err()
        );
    }

    #[test]
    fn test_command_required_with_dashes() {
        assert!(Cli::try_parse_from([
            "",
            "--slug",
            "no command after the dashes",
            "--",
        ])
        .is_err());
    }

    #[test]
    fn test_verbose_conflicts_with_quiet() {
        let base = ["", "--slug=test"];
        assert!(Cli::try_parse_from(
            base.iter().chain(["-v", "fake_command"].iter())
        )
        .is_ok());
        assert!(Cli::try_parse_from(
            base.iter().chain(["-q", "fake_command"].iter())
        )
        .is_ok());
        assert!(Cli::try_parse_from(
            base.iter().chain(["-q", "-v", "fake_command"].iter())
        )
        .is_err());
    }
}
