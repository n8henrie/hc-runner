# runner

master: [![master branch build status](https://github.com/n8henrie/runner-rs/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/n8henrie/runner-rs/actions/workflows/ci.yml)

<!-- dev: [![dev branch build status](https://github.com/n8henrie/runner-rs/actions/workflows/ci.yml/badge.svg?branch=dev)](https://github.com/n8henrie/runner-rs/actions/workflows/ci.yml) -->

A personal project that runs commands and submits the execution result to a
configurable instance of [healthchecks.io] (>=v3, requires auto-provisioning).

Not to be confused with <https://github.com/stevedonovan/runner>.

Either the environment variable `HC_RUNNER_URL` or the `--url` flag is required
at runtime, which specifies the slug-based ping URL for your healthchecks
server. If you're using the hosted healthchecks server, this may look something
like `https://hc-ping.com/{ping_key}/`.

\* Per [healthchecks.io], one's ping key should remain secret. If the `runner`
binary is world-readable, embedded strings could be extracted by a malicious
user. Consider `chmod 0700` or similar mitigations.

## Quickstart

`runner`:

- requires that the first argument be the "slug" by which this script's
execution should be identified at [healthchecks.io].
- interprets the remainder of the arguments as the command to be run and its
  arguments
- sends a request to `/start` to mark the beginning of the scripts execution
  and uses `?create=1` here to create a new healthcheck for this slug if it
  doesn't already exist
- sends a request to `/{status_code}` to mark the end of execution and reflect
  the exit status (e.g. `/0` for successful exit)
- echos all stdout and stderr to these respective streams
- can provide (a little) additional debugging with e.g. `RUST_LOG=debug`

Example:

```console
$ git clone https://github.com/n8henrie/runner-rs.git
$ cd runner-rs
$ export HC_RUNNER_URL=your.server.url
$ cargo build --release
$ ./target/release/runner say_foo echo foo
foo
$ echo $?
0
$ ./target/release/runner fail bash -c 'echo bar >/dev/stderr; exit 1'
bar
$ echo $?
1
```

## Notes

### testing

The integration tests use the `httpmock` library to provide a mock server.
Testing should be done with `--test-threads=1` or errors will likely result. The
`make test` target sets this for you.

### macOS

On macOS, `runner` prefixes commands with `caffeinate` in order to keep
long-running processes awake.

Newer versions of macOS have built-in privacy and security tools that may
prevent `runner` from accessing sensitive directories like `~/Documents`,
particularly if run in an automated script from `launchd`. The `install-macos`
target in the `Makefile` includes a workaround that should read `RUNNER_URL`
from `config.env` (see `config-sample.env`), compile and install the project,
then present some permissions dialogs to allow you to give access to these
directories to `runner`. If the scripts you are running do *not* access
sensitive directories such as `~/Desktop`, `~/Documents`, `~/Downloads`, don't
bother with this. If you *do* use this approach, you'll have to remove a check
named `runner-rs-setup-delete-me` from your [healthchecks.io] instance.

## Acknowledgements

- Pēteris Caune, creator of [healthchecks.io]!

[healthchecks.io]: https://healthchecks.io
[1]: https://healthchecks.io/docs/http_api/#start-slug
