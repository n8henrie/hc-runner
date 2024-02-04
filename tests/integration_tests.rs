use std::{env, error, process, result, str};

use httpmock::prelude::*;
use httpmock::Method::HEAD;

type Error = Box<dyn error::Error + Send + Sync>;
type Result<T> = result::Result<T, Error>;

const EXE: &str = env!("CARGO_BIN_EXE_hc-runner");

fn setup_server(ignore: bool) -> httpmock::MockServer {
    let server: httpmock::MockServer = httpmock::MockServer::start();

    if ignore {
        server.mock(|when, then| {
            when.any_request();
            then.status(200);
        });
    }

    env::set_var("RUNNER_URL", dbg!(server.url("")));
    server
}

#[test]
fn catches_stdout() -> Result<()> {
    setup_server(true);
    let result = process::Command::new(EXE)
        .args(["_", "echo", "-n", "foo"])
        .output()?;
    assert_eq!(str::from_utf8(&result.stdout)?, "foo");
    assert!(result.status.success());
    Ok(())
}

#[test]
fn catches_stderr() -> Result<()> {
    setup_server(true);
    let result = process::Command::new(EXE)
        .args(["_", "grep", "foo", "bar"])
        .output()?;

    let stderr = str::from_utf8(&result.stderr)?;
    assert!(stderr
        .trim()
        .lines()
        .next()
        .unwrap()
        .ends_with("No such file or directory"));
    assert!(!result.status.success());
    Ok(())
}

#[test]
fn catches_stdout_and_stderr() -> Result<()> {
    setup_server(true);
    let result = process::Command::new(EXE)
        .args([
            "_",
            "/bin/bash",
            "-c",
            "echo foo > /dev/stdout; echo bar > /dev/stderr",
        ])
        .output()?;

    assert_eq!(str::from_utf8(&result.stdout)?.trim(), "foo");
    assert_eq!(
        str::from_utf8(&result.stderr)?
            .lines()
            .next()
            .unwrap()
            .trim(),
        "bar"
    );
    assert!(result.status.success());
    Ok(())
}

#[test]
fn propagates_success() -> Result<()> {
    setup_server(true);
    let status = process::Command::new(EXE).args(["_", "true"]).status()?;
    assert!(status.success());
    Ok(())
}

#[test]
fn propagates_error() -> Result<()> {
    setup_server(true);
    let status = process::Command::new(EXE).args(["_", "false"]).status()?;
    assert!(!status.success());
    Ok(())
}

#[test]
fn calls_server_success() -> Result<()> {
    let server = setup_server(false);

    let mock_start = server.mock(|when, then| {
        when.method(HEAD)
            .path_matches(Regex::new("/winner/start$").unwrap())
            .query_param("create", "1");
        then.status(200);
    });
    let mock_end = server.mock(|when, then| {
        when.method(POST)
            .path_matches(Regex::new("/winner/0$").unwrap());
        then.status(200);
    });

    let status = process::Command::new(EXE)
        .args(["winner", "echo", "hooray!"])
        .status()?;
    mock_start.assert();
    mock_end.assert();
    assert!(status.success());
    Ok(())
}

#[test]
fn calls_server_error() -> Result<()> {
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
        .args(["failer", "bash", "-c", "echo whups > /dev/stderr; exit 7"])
        .status()?;
    mock_start.assert();
    mock_end.assert();
    assert!(!status.success());
    Ok(())
}
