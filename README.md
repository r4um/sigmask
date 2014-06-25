sigmask
=======

Decode and print process signal masks, given a process id on Linux. 
Decodes signal masks (SigCgt, SigIgn, SigBlk) in `/proc/PID/status`

To build
```
$ go build sigmask.go
```

Usage

```
$ Usage: ./sigmask [flags] pid
  -blocked=false: Show blocked
  -caught=false: Show caught
  -ignored=false: Show ignored
  -noname=false: Do not print signal name
```

Example
```
$ ./sigmask -blocked -caught -ignored $$
SigCgt 32,SIGVTALRM,SIGPROF,SIGIO,SIGXFSZ,SIGCONT,SIGALRM,SIGTERM,SIGSEGV,SIGUSR2,SIGPIPE,SIGFPE,SIGKILL,SIGABRT,SIGBUS,SIGTRAP,SIGINT,SIGQUIT
SigIgn SIGTTOU,SIGURG,SIGTTIN,SIGSTKFLT,SIGILL
SigBlk SIGCONT
```
