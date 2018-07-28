ERLS
=====

Manage multiple Erlang installs with per directory configuration.

## Build

```
$ cargo build --release
```

## Install

```
$ mkdir -p ~/.cache/erls/bin
$ cp ./target/release/erls ~/.cache/erls/bin/
$ export PATH=~/.cache/erls/bin:$PATH
```

## Build Erlang

First create a config file `~/.config/erls/config`:

```
[erls]
dir=<your home>/.cache/erls

[repos]
default=https://github.com/erlang/otp
```

Then fetch the default repo and build a version:

```
$ erls fetch
==> Fetching tags from https://github.com/erlang/otp
$ erls build OTP-21.0.4
...
$ erls default OTP-21.0.4
```

