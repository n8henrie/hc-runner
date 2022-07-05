# runner

master: [![master branch build status](https://github.com/n8henrie/runner-rs/actions/workflows/build.yml/badge.svg?branch=master)](https://github.com/n8henrie/runner-rs/actions/workflows/build.yml)

<!-- dev: [![dev branch build status](https://github.com/n8henrie/runner-rs/actions/workflows/build.yml/badge.svg?branch=dev)](https://github.com/n8henrie/runner-rs/actions/workflows/build.yml) -->

A personal project that runs commands and submits the execution result to a
local URL, for building a local dashboard for my automations. Kind of like a
toy version of <https://healthchecks.io/>.


Not to be confused with <https://github.com/stevedonovan/runner>.

## Quickstart

```console
$ git clone https://github.com/n8henrie/runner-rs.git
$ cd runner-rs
$ export URL=your.server.url
$ cargo build --release
$ ./target/release/runner say_foo echo foo
```
