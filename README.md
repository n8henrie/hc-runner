# runner

master: [![master branch build status](https://github.com/n8henrie/runner-rs/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/n8henrie/runner-rs/actions/workflows/ci.yml)

A personal project that runs commands and submits the execution result to a
configurable instance of [healthchecks.io] (>=v3, requires auto-provisioning).

The URL for your HealthChecks instance, including your `ping_key`, is required
at runtime\*. Because the `ping_key` is considered a secret, users may wish to
keep it out of their shell history and out of any cron scripts that are calling
`hc-runner`; to this end, as an alternative to the `--url` flag, the URL can
also be specified in a config file or by the `HC_RUNNER_URL` environment
variable. All other options are taken only from command line flags.

Please consider restricting access (e.g. `chmod 0600`) to any files that
contain your `ping_key`, possibly including the `hc-runner` config file.

\* If you're using the hosted healthchecks server, your URL may look something
like `https://hc-ping.com/{ping_key}/`.

## Quickstart

```console
$ cargo run -q -- --help
Command runner for healthchecks.io

Usage: hc-runner [OPTIONS] --slug <NAME> <COMMAND>...

Arguments:
  <COMMAND>...

Options:
  -q, --quiet              Silence logging / warnings. Does not affect called command's output
  -s, --slug <NAME>        Set healthchecks slug for this call
      --success-only       Disable calling `/start` and only ping healthchecks if the test was successful
  -t, --timeout <TIMEOUT>  Set timeout for requests to healthchecks server [default: 10]
  -u, --url <URL>          Specify the URL of the healthchecks server for this call
  -v, --verbose...         Increase logging verbosity. May be repeated. Defaults to `Level::WARN`
  -h, --help               Print help
  -V, --version            Print version
```

`runner`:

- by default sends a request to `/start` to mark the beginning of the scripts
  execution and uses `?create=1` here to create a new healthcheck for this slug
  if it doesn't already exist
- by default sends a request to `/{status_code}` to mark the end of execution
  and reflect the exit status (e.g. `/0` for successful exit) and sends stderr
  as the body
- mirrors the exit status, stdout, and stderr of the called command
- can optionally only report successful runs with `--success-only`
    - this will prevent failure notifications for services that are expected to
      fail *sometimes*, but for which notifications are still desired if there
      isn't at least one successful run per (healthchecks-configured) time
      period
    - does not report execution time or collect stderr
- can disambiguate flags in the called command using `-- trailing args` syntax,
  e.g.:
    - `hc-runner -v -- command` makes `hc-runner` more verbose
    - `hc-runner -- command -v` passes the `-v` flag to `command`
    - `hc-runner -v -- command -v` does both
- prepends commands with `/usr/bin/caffeinate` on MacOS to keep long-running
  commands alive

Example:

```console
$ git clone https://github.com/n8henrie/runner-rs.git
$ cd runner-rs
$ export HC_RUNNER_URL=http://your.server.url
$ cargo build --release
$ ./target/release/runner --slug say_foo -- echo foo
foo
$ echo $?
0
$ ./target/release/runner \
    --slug epic_fail \
    -- bash -c 'echo bar >/dev/stderr; exit 1'
bar
$ echo $?
1
```

## Notes

### debugging

`-vvv` is your friend. Note that the output will contain your `ping_key`.

Due to the default of `create=1`, you will pollute your HealthChecks instance
when testing with fake slugs (`--slug=foo`), but your output will be cluttered
with errors if you use a fake URL (`--url=http://broken`). During testing /
experimentation, consider using `--success-only` with a command that is
guaranteed to fail (e.g. `false`) which will prevent calls to the server
entirely.

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

### Alternatives

There are several similar projects on crates.io that may be much more
comprehensive and/or functional than this hobby project. I encourage you to
check them out! Here are a couple:

- https://github.com/msfjarvis/healthchecks-rs
- https://github.com/dimo414/task-mon

# Acknowledgements

- Pēteris Caune, creator of [healthchecks.io]!

[healthchecks.io]: https://healthchecks.io
[1]: https://healthchecks.io/docs/http_api/#start-slug
