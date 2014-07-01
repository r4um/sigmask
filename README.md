sigmask
=======

Decode and print process signal masks, given a process id on Linux.
Decodes signal masks (SigCgt, SigIgn, SigBlk, ShdPnd, SigPnd) in `/proc/PID/status`

To build
```
$ go build sigmask.go
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
$ ./sigmask -blocked -caught -ignored -pending -shpending $$
SigCgt 32,SIGVTALRM,SIGPROF,SIGIO,SIGXFSZ,SIGCONT,SIGALRM,SIGTERM,SIGSEGV,SIGUSR2,SIGPIPE,SIGFPE,SIGKILL,SIGABRT,SIGBUS,SIGTRAP,SIGINT,SIGQUIT
SigIgn SIGTTOU,SIGURG,SIGTTIN,SIGSTKFLT,SIGILL
SigBlk SIGCONT
SigPnd
ShdPnd
```
