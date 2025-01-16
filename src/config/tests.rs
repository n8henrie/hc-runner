use std::sync::{LazyLock, Mutex};
use std::{env, fs};

use tempfile::tempdir;

use super::*;

static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(Mutex::default);

/// Returns the `TempDir` to prevent destruction at the end of the function
fn temp_config(contents: impl AsRef<str>) -> tempfile::TempDir {
    let home = tempdir().unwrap();
    env::set_var("HOME", home.path());
    env::remove_var("XDG_CONFIG_HOME");

    let suffix = if cfg!(target_os = "macos") {
        "Library/Application Support/com.n8henrie.hc-runner/config.toml"
    } else if cfg!(target_os = "linux") {
        ".config/hc-runner/config.toml"
    } else {
        panic!("Testing not (yet) supported on your platform. Contributions appreciated!");
    };

    let path = home.path().to_path_buf().join(suffix);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents.as_ref()).unwrap();
    home
}

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
    assert!(Cli::try_parse_from(["", "--slug=no_command_no_dashes"]).is_err());
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

#[test]
fn test_specify_config_file() {
    let cli = Cli::parse_from(["", "--slug=test", "fake_command"]);
    assert_eq!(cli.config, None);
    let cli = Cli::parse_from([
        "",
        "--slug=test",
        "--config=/dev/null",
        "fake_command",
    ]);
    assert_eq!(cli.config, Some("/dev/null".into()));
}

#[test]
fn test_timeout_overrides() {
    let env_guard = ENV_LOCK.lock().unwrap();
    // remove confounding environment
    env::remove_var("HC_RUNNER_TIMEOUT");
    env::set_var("HOME", "/dev/null");

    // test defaults
    let cli = Cli::parse_from([
        "",
        "--url=https://n8henrie.com",
        "--slug=test",
        "fake_command",
    ]);
    let config = Config::resolve_with(cli.clone()).unwrap();
    assert_eq!(config.timeout, 10);

    // test override with file config
    let _tmp = temp_config(r#"timeout = "20""#);
    let config = Config::resolve_with(cli.clone()).unwrap();
    assert_eq!(config.timeout, 20);

    // test override with env
    env::set_var("HC_RUNNER_TIMEOUT", "30");
    let config = Config::resolve_with(cli).unwrap();
    assert_eq!(config.timeout, 30);

    // test override with cli
    let cli = Cli::parse_from([
        "",
        "--url=https://n8henrie.com",
        "--slug=test",
        "--timeout=40",
        "fake_command",
    ]);
    let config = Config::resolve_with(cli).unwrap();
    assert_eq!(config.timeout, 40);

    drop(env_guard);
}
