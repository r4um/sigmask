sigmask
=======

[![crates.io](https://img.shields.io/crates/v/sigmask.svg)](https://crates.io/crates/sigmask)
[![docs.rs](https://docs.rs/sigmask/badge.svg)](https://docs.rs/sigmask)

Decode and print process signal masks, given a process id on Linux.
Decodes signal masks (SigCgt, SigIgn, SigBlk, ShdPnd, SigPnd) in `/proc/PID/status`

Install
-------

From crates.io:

```
$ cargo install sigmask
```

Or download a static Linux binary (x86_64 or aarch64) from the
[releases page](https://github.com/r4um/sigmask/releases).

Usage
-----

```
$ sigmask --help
Decode and print process signal masks from /proc/PID/status

Usage: sigmask [OPTIONS] [PID|PATH]

Arguments:
  [PID|PATH]  Process id, or path to a proc status file ("-" or piped input reads stdin)

Options:
  -p, --pending      Show pending (SigPnd)
  -s, --shpending    Show shared pending (ShdPnd)
  -b, --blocked      Show blocked (SigBlk)
  -i, --ignored      Show ignored (SigIgn)
  -c, --caught       Show caught (SigCgt)
  -m, --mask <MASK>  Decode a single hex mask instead of reading a status file
  -n, --noname       Print signal numbers instead of names
  -h, --help         Print help
  -V, --version      Print version
```

Examples

```
$ sigmask $$
SigPnd 0000000000000000
ShdPnd 0000000000000000
SigBlk 0000000000010000 SIGCHLD
SigIgn 0000000000384004 SIGQUIT,SIGTERM,SIGTSTP,SIGTTIN,SIGTTOU
SigCgt 000000004b813efb SIGHUP,SIGINT,SIGILL,SIGTRAP,SIGABRT,SIGBUS,SIGFPE,SIGUSR1,SIGSEGV,SIGUSR2,SIGPIPE,SIGALRM,SIGCHLD,SIGXCPU,SIGXFSZ,SIGVTALRM,SIGWINCH,SIGSYS
$ sigmask 1
SigPnd 0000000000000000
ShdPnd 0000000000000000
SigBlk 7fefc1fe28014a03 SIGHUP,SIGINT,SIGUSR1,SIGUSR2,SIGTERM,SIGCHLD,SIGWINCH,SIGPWR,SIGRTMIN,SIGRTMIN+1,SIGRTMIN+2,SIGRTMIN+3,SIGRTMIN+4,SIGRTMIN+5,SIGRTMIN+6,SIGRTMIN+7,SIGRTMIN+13,SIGRTMIN+14,SIGRTMIN+15,SIGRTMAX-14,SIGRTMAX-13,SIGRTMAX-12,SIGRTMAX-10,SIGRTMAX-9,SIGRTMAX-8,SIGRTMAX-7,SIGRTMAX-6,SIGRTMAX-5,SIGRTMAX-4,SIGRTMAX-3,SIGRTMAX-2,SIGRTMAX-1
SigIgn 0000000000001000 SIGPIPE
SigCgt 00000000000004ec SIGQUIT,SIGILL,SIGABRT,SIGBUS,SIGFPE,SIGSEGV
$ sigmask --caught --noname 1
SigCgt 00000000000004ec 3,4,6,7,8,11
$ cat /proc/1/status | sigmask --blocked
$ sigmask --mask 0000000000384004
SIGQUIT,SIGTERM,SIGTSTP,SIGTTIN,SIGTTOU
```

Upgrading from the Go version: flags now use two dashes (`--mask=MASK`,
`--noname`, `--caught`, ...) and have short forms (`-m`, `-n`, `-c`, ...).
The Go implementation is kept on the `go` branch.

Library
-------

```toml
[dependencies]
sigmask = { version = "0.1", default-features = false }
```

`default-features = false` drops the CLI's `clap` dependency.

```rust
use sigmask::{MaskKind, ProcStatus, format_signals};

let status = ProcStatus::from_pid(1)?;
let caught = status.decode(MaskKind::Caught)?;
println!("{}", format_signals(&caught, true));
```

Releasing
---------

1. Bump `version` in `Cargo.toml`, run `cargo build` to update `Cargo.lock`, and commit.
2. `git tag vX.Y.Z && git push origin vX.Y.Z`

The release workflow checks that the tag matches `Cargo.toml`, builds static
musl binaries for x86_64 and aarch64, attaches them to a GitHub release and
publishes the crate to crates.io via trusted publishing.
