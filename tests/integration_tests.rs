use std::sync::{LazyLock, Mutex};
use std::{env, fs, process, str};

use httpmock::prelude::*;
use httpmock::{Method::HEAD, Mock};
use tempfile::{tempdir, Builder};

const EXE: &str = env!("CARGO_BIN_EXE_hc-runner");

static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(Mutex::default);

fn setup_server(ignore: bool) -> httpmock::MockServer {
    let server: httpmock::MockServer = httpmock::MockServer::start();

    if ignore {
        server.mock(|when, then| {
            when.any_request();
            then.status(200);
        });
    }

    // Prevent reading config from my actual filesystem.
    // Unsetting `HOME` doesn't work due to this `fallback`:
    // https://github.com/dirs-dev/dirs-sys-rs/blob/8bcd4aa2c35990d57a2cff2953793525fc42709c/src/lib.rs#L44
    env::set_var("HOME", "/dev/null");

    server
}

#[test]
fn catches_stdout() {
    let server = setup_server(true);
    let result = process::Command::new(EXE)
        .args([
            "--slug=_",
            "--url",
            &server.url(""),
            "--",
            "echo",
            "-n",
            "foo",
        ])
        .output()
        .unwrap();
    assert_eq!(str::from_utf8(&result.stdout).unwrap(), "foo");
    assert!(result.status.success());
}

#[test]
fn catches_stderr() {
    let server = setup_server(true);
    let result = process::Command::new(EXE)
        .args(["--slug=_", "--url", &server.url(""), "grep", "foo", "bar"])
        .output()
        .unwrap();

    let stderr = str::from_utf8(&result.stderr).unwrap();
    assert!(stderr
        .trim()
        .lines()
        .next()
        .unwrap()
        .ends_with("No such file or directory"));
    assert!(!result.status.success());
}

fn successful_run<'a>(
    server: &'a MockServer,
    slug: &str,
) -> (Mock<'a>, Mock<'a>) {
    let mock_start = server.mock(|when, then| {
        when.method(HEAD)
            .path_matches(
                Regex::new(format!("/{slug}/start$").as_ref()).unwrap(),
            )
            .query_param("create", "1");
        then.status(200);
    });
    let mock_end = server.mock(|when, then| {
        when.method(POST)
            .path_matches(Regex::new(format!("/{slug}/0$").as_ref()).unwrap());
        then.status(200);
    });
    (mock_start, mock_end)
}

#[test]
fn catches_stdout_and_stderr() {
    let server = setup_server(true);
    let result = process::Command::new(EXE)
        .args([
            "--slug=_",
            "--url",
            &server.url(""),
            "bash",
            "-c",
            "echo foo > /dev/stdout; echo bar > /dev/stderr",
        ])
        .output()
        .unwrap();

    assert_eq!(str::from_utf8(&result.stdout).unwrap().trim(), "foo");
    assert_eq!(
        str::from_utf8(&result.stderr)
            .unwrap()
            .lines()
            .next()
            .unwrap()
            .trim(),
        "bar"
    );
    assert!(result.status.success());
}

#[test]
fn propagates_success() {
    let server = setup_server(true);
    let status = process::Command::new(EXE)
        .args(["--slug=_", "--url", &server.url(""), "true"])
        .output()
        .unwrap()
        .status;
    assert!(status.success());
}

#[test]
fn propagates_error() {
    let server = setup_server(true);
    let status = process::Command::new(EXE)
        .args(["--slug=_", "--url", &server.url(""), "false"])
        .output()
        .unwrap()
        .status;
    assert!(!status.success());
}

#[test]
fn calls_server_success() {
    let server = setup_server(false);
    let (mock_start, mock_end) = successful_run(&server, "winner");

    let status = process::Command::new(EXE)
        .args(["--slug=winner", "--url", &server.url(""), "echo", "hooray!"])
        .output()
        .unwrap()
        .status;
    mock_start.assert();
    mock_end.assert();
    assert!(status.success());
}

#[test]
fn calls_server_error() {
    let server = setup_server(false);

    let mock_start = server.mock(|when, then| {
        when.method(HEAD)
            .path_matches(Regex::new("/failer/start$").unwrap())
            .query_param("create", "1");
        then.status(200);
    });
    let mock_end = server.mock(|when, then| {
        when.method(POST)
            .path_matches(Regex::new("/failer/7$").unwrap());
        then.status(200);
    });

    let status = process::Command::new(EXE)
        .args([
            "--slug=failer",
            "--url",
            &server.url(""),
            "bash",
            "-c",
            "echo whups > /dev/stderr; exit 7",
        ])
        .output()
        .unwrap()
        .status;
    mock_start.assert();
    mock_end.assert();
    assert!(!status.success());
}

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
fn file_config_works() {
    let server = setup_server(false);
    let (mock_start, mock_end) = successful_run(&server, "winner");

    let mut cmd = process::Command::new(EXE);
    let cmd = cmd.args(["--slug=winner", "echo", "hooray!"]);

    let env_guard = ENV_LOCK.lock().unwrap();
    env::remove_var("HC_RUNNER_URL");

    // Confirm failure with the env_var unset
    let status = cmd.output().unwrap().status;
    mock_start.assert_hits(0);
    mock_end.assert_hits(0);
    assert!(!status.success());

    // Should work again obtaining the URL from the config file
    let _tmp = temp_config(format!(r#"url = "{}""#, server.url("")));
    let status = cmd.output().unwrap().status;
    drop(env_guard);

    mock_start.assert_hits(1);
    mock_end.assert_hits(1);
    assert!(status.success());
}

#[test]
fn env_works() {
    let server = setup_server(false);
    let (mock_start, mock_end) = successful_run(&server, "winner");

    let mut cmd = process::Command::new(EXE);
    let cmd = cmd.args(["--slug=winner", "echo", "hooray!"]);

    let env_guard = ENV_LOCK.lock().unwrap();

    // Ensure this is not set, verify failure in absence
    env::remove_var("HC_RUNNER_URL");
    let status = cmd.output().unwrap().status;
    mock_start.assert_hits(0);
    mock_end.assert_hits(0);
    assert!(!status.success());

    env::set_var("HC_RUNNER_URL", server.url(""));
    let status = cmd.output().unwrap().status;
    drop(env_guard);

    mock_start.assert_hits(1);
    mock_end.assert_hits(1);
    assert!(status.success());
}

// This tests that the `--url` flag overrides the envvar default.
#[test]
fn flag_overrides_env() {
    let server = setup_server(false);
    let (mock_start, mock_end) = successful_run(&server, "winner");

    let mut args = vec!["--slug=winner", "echo", "hooray!"];
    let cmd = |args| {
        process::Command::new(EXE)
            .args(args)
            .output()
            .unwrap()
            .status
    };

    let env_guard = ENV_LOCK.lock().unwrap();
    env::set_var("HC_RUNNER_URL", "http://broken");

    let status = cmd(args.clone());
    mock_start.assert_hits(0);
    mock_end.assert_hits(0);
    assert!(!status.success());

    let url_flag = format!("--url={}", &server.url(""));
    args.insert(0, url_flag.as_ref());
    let status = cmd(args);
    drop(env_guard);

    mock_start.assert_hits(1);
    mock_end.assert_hits(1);
    assert!(status.success());
}

#[test]
fn env_overrides_file() {
    let server = setup_server(false);
    let (mock_start, mock_end) = successful_run(&server, "winner");

    let mut cmd = process::Command::new(EXE);
    let cmd = cmd.args(["--slug=winner", "echo", "hooray!"]);

    let env_guard = ENV_LOCK.lock().unwrap();
    env::remove_var("HC_RUNNER_URL");

    // Set a broken url in the config file, failure shows it was used
    let _tmp = temp_config(r#"url = "http://broken""#);
    let status = cmd.output().unwrap().status;
    mock_start.assert_hits(0);
    mock_end.assert_hits(0);
    assert!(!status.success());

    // Confirm settings the envvar overrides the bad config
    env::set_var("HC_RUNNER_URL", server.url(""));
    let status = cmd.output().unwrap().status;
    drop(env_guard);

    mock_start.assert_hits(1);
    mock_end.assert_hits(1);
    assert!(status.success());
}

#[test]
fn specify_config_file() {
    let server = setup_server(false);
    let (mock_start, mock_end) = successful_run(&server, "winner");

    let env_guard = ENV_LOCK.lock().unwrap();
    env::remove_var("HC_RUNNER_URL");

    // Confirm failure with the env_var unset
    let status = process::Command::new(EXE)
        .args(["--slug=winner", "echo", "hooray!"])
        .output()
        .unwrap()
        .status;
    mock_start.assert_hits(0);
    mock_end.assert_hits(0);
    assert!(!status.success());

    // Write config (with the server url) to a file and specify that as config
    let config = Builder::new().suffix(".toml").tempfile().unwrap();
    fs::write(config.path(), format!(r#"url = "{}""#, server.url("")))
        .unwrap();

    let status = process::Command::new(EXE)
        .args([
            "--config",
            config.path().to_str().unwrap(),
            "--slug=winner",
            "echo",
            "hooray!",
        ])
        .output()
        .unwrap()
        .status;
    drop(env_guard);
    mock_start.assert_hits(1);
    mock_end.assert_hits(1);
    assert!(status.success());
}
