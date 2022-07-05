use std::{env, error, process, result, str};

type Error = Box<dyn error::Error + Send + Sync>;
type Result<T> = result::Result<T, Error>;

const EXE: &str = env!("CARGO_BIN_EXE_runner");

fn setup_server(ignore: bool) -> httpmock::MockServer {
    let server: httpmock::MockServer = httpmock::MockServer::start();

    if ignore {
        server.mock(|when, then| {
            when.any_request();
            then.status(200);
        });
    }

    env::set_var("URL", dbg!(server.url("")));
    server
}

#[test]
fn catches_stdout() -> Result<()> {
    setup_server(true);
    let result = process::Command::new(EXE)
        .args(&["_", "echo", "-n", "foo"])
        .output()?;
    assert_eq!(str::from_utf8(&result.stdout)?, "foo");
    assert!(result.status.success());
    Ok(())
}

#[test]
fn catches_stderr() -> Result<()> {
    setup_server(true);
    let result = process::Command::new(EXE)
        .args(&["_", "grep", "foo", "bar"])
        .output()?;

    let stderr = str::from_utf8(&result.stderr)?;
    assert!(stderr.trim().ends_with("No such file or directory"));
    assert!(!result.status.success());
    Ok(())
}

#[test]
fn catches_stdout_and_stderr() -> Result<()> {
    setup_server(true);
    let result = process::Command::new(EXE)
        .args(&[
            "_",
            "/bin/bash",
            "-c",
            "echo foo > /dev/stdout; echo bar > /dev/stderr",
        ])
        .output()?;

    assert_eq!(str::from_utf8(&result.stdout)?, "foo\n");
    assert_eq!(str::from_utf8(&result.stderr)?, "bar\n");
    assert!(result.status.success());
    Ok(())
}

#[test]
fn propagates_success() -> Result<()> {
    setup_server(true);
    let status = process::Command::new(EXE).args(&["_", "true"]).status()?;
    assert!(status.success());
    Ok(())
}

#[test]
fn propagates_error() -> Result<()> {
    setup_server(true);
    let status = process::Command::new(EXE).args(&["_", "false"]).status()?;
    assert!(!status.success());
    Ok(())
}

#[test]
fn calls_server_success() -> Result<()> {
    use runner::RunnerMessage;
    let server = setup_server(false);

    let runner_mock = server.mock(|when, then| {
        when.json_body_obj(&RunnerMessage {
            name: String::from("winner"),
            // Only stderr is captured
            message: String::from(""),
            exit_code: 0,
        });
        then.status(200);
    });

    let status = process::Command::new(EXE)
        .args(&["winner", "echo", "hooray!"])
        .status()?;
    runner_mock.assert();
    assert!(status.success());
    Ok(())
}

#[test]
fn calls_server_error() -> Result<()> {
    use runner::RunnerMessage;

    let server = setup_server(false);

    let runner_mock = server.mock(|when, then| {
        when.json_body_obj(&RunnerMessage {
            name: String::from("failer"),
            message: String::from("whups\n"),
            exit_code: 7,
        });
        then.status(200);
    });

    let status = process::Command::new(EXE)
        .args(&["failer", "bash", "-c", "echo whups > /dev/stderr; exit 7"])
        .status()?;
    runner_mock.assert();
    assert!(!status.success());
    Ok(())
}
