use std::borrow::Cow;

use crate::Result;
use clap::builder::NonEmptyStringValueParser;
use clap::Parser;
use reqwest::Url;

pub(crate) fn parse_url(s: &str) -> Result<Url> {
    // subsequent calls to `join` will only interpret the last segment of the
    // path as a directory if it has a trailing slash
    // https://docs.rs/reqwest/latest/reqwest/struct.Url.html#method.join
    let url = if s.ends_with('/') {
        Cow::Borrowed(s)
    } else {
        Cow::Owned(String::from(s) + "/")
    };
    Ok(Url::parse(url.as_ref())?)
}

#[derive(Debug, Parser)]
#[command(author, version, about, about, long_about)]
pub struct Config {
    #[arg(trailing_var_arg(true), required(true), value_parser=NonEmptyStringValueParser::new())]
    pub(crate) command: Vec<String>,

    /// Silence warnings
    #[arg(short, long, conflicts_with("verbose"))]
    pub quiet: bool,

    #[arg(short, long, value_name = "NAME", value_parser=NonEmptyStringValueParser::new())]
    pub(crate) slug: String,

    /// Disable calling `/start` and only ping healthchecks if the test was successful
    #[arg(long)]
    pub(crate) success_only: bool,

    /// Timeout for requests to healthchecks server
    #[arg(short, long, default_value("10"))]
    pub(crate) timeout: u64,

    /// Specify the URL of the healthchecks server for this call
    #[arg(short, long, value_parser=parse_url)]
    pub(crate) url: Option<Url>,

    /// Increase logging verbosity. May be repeated. Defaults to `Level::WARN`
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parser() {
        let config = Config::parse_from([
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
            Config::parse_from(["", "--slug=fake", "cat"]),
            Config::parse_from(["", "--slug", "fake", "--", "cat"]),
            Config::parse_from(["", "-s", "fake", "cat"]),
        ] {
            assert_eq!(config.slug, "fake");
            assert_eq!(config.success_only, false);
            assert_eq!(config.command, vec!["cat"]);
        }
    }

    #[test]
    fn test_command_required() {
        assert!(Config::try_parse_from(["", "--slug=no_command_no_dashes"])
            .is_err())
    }

    #[test]
    fn test_command_required_with_dashes() {
        assert!(Config::try_parse_from([
            "",
            "--slug",
            "no command after the dashes",
            "--",
        ])
        .is_err());
    }

    #[test]
    fn test_verbose_conflicts_with_quiet() {
        let base = vec!["", "--slug=test"];
        assert!(Config::try_parse_from(
            base.iter().chain(["-v", "fake_command"].iter())
        )
        .is_ok());
        assert!(Config::try_parse_from(
            base.iter().chain(["-q", "fake_command"].iter())
        )
        .is_ok());
        assert!(Config::try_parse_from(
            base.iter().chain(["-q", "-v", "fake_command"].iter())
        )
        .is_err());
    }
}
