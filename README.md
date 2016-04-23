ERLS
=====

Manage multiple Erlang installs with per directory configuration.

## Build

```
$ cargo build --release
```

## Install

```
$ mkdir -p ~/.erls/bin
$ cp ./target/release/erls ~/.erls/bin/
$ export PATH=~/.erls/bin:$PATH
```

## Build Latest Stable Erlang

```
$ erls add repo https://github.com/erlang/otp
$ erls build latest
```
