package main

import (
	"encoding/csv"
	"flag"
	"fmt"
	"io"
	"os"
	"strconv"
	"strings"
)

// Generated from
// kill -l | tr "\t" "\n" | ruby -ne '$_.scan(/(\d+)\) (.*)/) {|f,k| puts "#{f}:\"#{k}\"," }'
var SIGNAMES = map[int]string{
	1:  "SIGHUP",
	2:  "SIGINT",
	3:  "SIGQUIT",
	4:  "SIGILL",
	5:  "SIGTRAP",
	6:  "SIGABRT",
	7:  "SIGBUS",
	8:  "SIGFPE",
	9:  "SIGKILL",
	10: "SIGUSR1",
	11: "SIGSEGV",
	12: "SIGUSR2",
	13: "SIGPIPE",
	14: "SIGALRM",
	15: "SIGTERM",
	16: "SIGSTKFLT",
	17: "SIGCHLD",
	18: "SIGCONT",
	19: "SIGSTOP",
	20: "SIGTSTP",
	21: "SIGTTIN",
	22: "SIGTTOU",
	23: "SIGURG",
	24: "SIGXCPU",
	25: "SIGXFSZ",
	26: "SIGVTALRM",
	27: "SIGPROF",
	28: "SIGWINCH",
	29: "SIGIO",
	30: "SIGPWR",
	31: "SIGSYS",
	34: "SIGRTMIN",
	35: "SIGRTMIN+1",
	36: "SIGRTMIN+2",
	37: "SIGRTMIN+3",
	38: "SIGRTMIN+4",
	39: "SIGRTMIN+5",
	40: "SIGRTMIN+6",
	41: "SIGRTMIN+7",
	42: "SIGRTMIN+8",
	43: "SIGRTMIN+9",
	44: "SIGRTMIN+10",
	45: "SIGRTMIN+11",
	46: "SIGRTMIN+12",
	47: "SIGRTMIN+13",
	48: "SIGRTMIN+14",
	49: "SIGRTMIN+15",
	50: "SIGRTMAX-14",
	51: "SIGRTMAX-13",
	52: "SIGRTMAX-12",
	53: "SIGRTMAX-11",
	54: "SIGRTMAX-10",
	55: "SIGRTMAX-9",
	56: "SIGRTMAX-8",
	57: "SIGRTMAX-7",
	58: "SIGRTMAX-6",
	59: "SIGRTMAX-5",
	60: "SIGRTMAX-4",
	61: "SIGRTMAX-3",
	62: "SIGRTMAX-2",
	63: "SIGRTMAX-1",
	64: "SIGRTMAX",
}

// Decode signal mask. See function render_sigset_t in kernel source fs/proc/array.c
func DecodeSigmask(mask string, nosigname bool) []string {
	bm := map[int]int{1: 1, 2: 2, 4: 3, 8: 4}
	signals := make([]string, 0)

	//echo "#include <signal.h>" | gcc -dM -E -  | grep define._NSIG
	_NSIG := 65

	pos := 1

	for _, m := range strings.Split(mask, "") {
		m_int, e := strconv.ParseInt(m, 16, 0)

		if e != nil {
			fmt.Fprintf(os.Stderr, "unable to convert mask %s entry to int\n", m)
			os.Exit(2)
		}

		i := _NSIG - pos*4

		for k, _ := range bm {
			if m_int&int64(k) == int64(k) {
				sig := i + bm[k]
				if nosigname {
					signals = append(signals, strconv.Itoa(sig))
				} else {
					if name, ok := SIGNAMES[sig]; ok {
						signals = append(signals, name)
					} else {
						signals = append(signals, strconv.Itoa(sig))
					}
				}
			}
		}
		pos = pos + 1
	}
	return signals
}

func Usage() {
	fmt.Fprintf(os.Stderr, "Usage: %s [flags] pid\n", os.Args[0])
	flag.PrintDefaults()
}

func main() {
	sigmasks := make(map[string]*bool)

	sigmasks["SigCgt"] = flag.Bool("caught", false, "Show caught")
	sigmasks["SigIgn"] = flag.Bool("ignored", false, "Show ignored")
	sigmasks["SigBlk"] = flag.Bool("blocked", false, "Show blocked")
	nosigname := flag.Bool("noname", false, "Do not print signal name")

	flag.Usage = Usage
	flag.Parse()

	args := flag.Args()

	if len(args) < 1 {
		fmt.Fprintf(os.Stderr, "missing process id\n")
		flag.Usage()
		os.Exit(1)
	}

	pid := args[0]
	pid_proc_status := fmt.Sprintf("/proc/%s/status", pid)

	file, err := os.Open(pid_proc_status)

	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %s\n", err)
		return
	}

	defer file.Close()

	reader := csv.NewReader(file)
	reader.Comma = ':'
	reader.FieldsPerRecord = 2

	pid_statuses := make(map[string]string)

	for {
		record, err := reader.Read()
		if err == io.EOF {
			break
		} else if err != nil {
			fmt.Fprintf(os.Stderr, "error: %s\n", err)
			return
		}
		pid_statuses[record[0]] = strings.TrimSpace(record[1])
	}

	for k, v := range sigmasks {
		if *v {
			fmt.Fprintf(os.Stdout, "%s %s\n", k, strings.Join(DecodeSigmask(pid_statuses[k], *nosigname), ","))
		}
	}
}
