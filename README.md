# runner

master: [![master branch build status](https://github.com/n8henrie/runner-rs/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/n8henrie/runner-rs/actions/workflows/ci.yml)

<!-- dev: [![dev branch build status](https://github.com/n8henrie/runner-rs/actions/workflows/ci.yml/badge.svg?branch=dev)](https://github.com/n8henrie/runner-rs/actions/workflows/ci.yml) -->

A personal project that runs commands and submits the execution result to a
local URL, for building a local dashboard for my automations. Kind of like a
toy version of <https://healthchecks.io/>.


Not to be confused with <https://github.com/stevedonovan/runner>.

## Quickstart

Of note, the environment variable `URL` is *compile-time* embedded into the
binary and in my case is a HomeAssistant webhook reporting the exit status of
the script (and a little contextual stdout / stderr type info).

```console
$ git clone https://github.com/n8henrie/runner-rs.git
$ cd runner-rs
$ export URL=your.server.url
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

Alternatively, see the Makefile.
