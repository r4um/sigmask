sigmask
=======

Decode and print process signal masks, given a process id on Linux.
Decodes signal masks (SigCgt, SigIgn, SigBlk, ShdPnd, SigPnd) in `/proc/PID/status`

To install/build and run.

```
$ go get -v github.com/r4um/sigmask
$ ${GOPATH:=~/go}/bin/sigmask
```

Usage
```
Usage: ./sigmask [flags] pid
Usage: ./sigmask [flags] proc_status_path
Usage: ./sigmask [-noname] -mask=MASK
  -blocked=false: Show blocked
  -caught=false: Show caught
  -ignored=false: Show ignored
  -mask="": Decode mask
  -noname=false: Do not print signal name
  -pending=false: Show pending
  -shpending=false: Show shared pending
```

Example
```
$ ./sigmask $$
SigPnd
ShdPnd
SigBlk SIGCHLD
SigIgn SIGQUIT,SIGTSTP,SIGTTIN,SIGTTOU
SigCgt SIGHUP,SIGINT,SIGILL,SIGTRAP,SIGABRT,SIGBUS,SIGFPE,SIGUSR1,SIGSEGV,SIGUSR2,SIGPIPE,SIGALRM,SIGTERM,SIGCHLD,SIGXCPU,SIGXFSZ,SIGVTALRM,SIGWINCH,SIGSYS
```
