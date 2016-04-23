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
==> Building Erlang OTP-18.3.1...
==> Build complete
```

```
$ erls list
18.3.1 -> $HOME/.erls/otps/OTP-18.3.1/dist
```

```
$ cat ~/.erls/config
[erls]
default=OTP-18.3.1
dir=$HOME/.erls

[erlangs]
OTP-18.3.1=$HOME/.erls/otps/OTP-18.3.1/dist

[repos]
default=https://github.com/erlang/otp
```
