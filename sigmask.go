package main

import (
	"encoding/csv"
	"flag"
	"fmt"
	"io"
	"math/big"
	"os"
	"strconv"
	"strings"

	"github.com/orivej/e"
)

// SIGNAMES Generated from
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
	32: "SIGRTMIN-2",
	33: "SIGRTMIN-1",
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

func ParseStatuses(r io.Reader) map[string]string {
	reader := csv.NewReader(r)
	reader.Comma = ':'
	reader.FieldsPerRecord = -1

	statusMap := make(map[string]string)

	for {
		record, err := reader.Read()
		if err == io.EOF {
			break
		}
		if err != nil {
			fmt.Fprintf(os.Stderr, "error: %s\n", err)
			return nil
		}
		statusMap[record[0]] = strings.TrimSpace(record[1])
	}

	return statusMap
}

// DecodeSigmask decodes signal mask. See function render_sigset_t in kernel
// source fs/proc/array.c
func DecodeSigmask(mask string, nosigname bool) string {
	var n big.Int

	_, ok := n.SetString(mask, 16)
	if !ok {
		fmt.Fprintf(os.Stderr, "unable to parse hex mask entry %s\n", mask)
		os.Exit(2)
	}

	var signals []string

	for i := 0; i < n.BitLen(); i++ {
		if n.Bit(i) == 1 {
			name, ok := SIGNAMES[i+1]
			if nosigname || !ok {
				name = strconv.Itoa(i + 1)
			}
			signals = append(signals, name)
		}
	}
	return strings.Join(signals, ",")
}

func Usage() {
	fmt.Fprintf(os.Stderr, "Usage: %s [flags] pid\n", os.Args[0])
	fmt.Fprintf(os.Stderr, "Usage: %s [flags] proc_status_path\n", os.Args[0])
	fmt.Fprintf(os.Stderr, "Usage: %s [-noname] -mask=MASK\n", os.Args[0])
	flag.PrintDefaults()
}

func main() {
	sigmasks := []struct {
		name     string
		selected *bool
	}{
		{"SigPnd", flag.Bool("pending", false, "Show pending")},
		{"ShdPnd", flag.Bool("shpending", false, "Show shared pending")},
		{"SigBlk", flag.Bool("blocked", false, "Show blocked")},
		{"SigIgn", flag.Bool("ignored", false, "Show ignored")},
		{"SigCgt", flag.Bool("caught", false, "Show caught")},
	}

	var mask string
	flag.StringVar(&mask, "mask", "", "Decode mask")

	var nosigname bool
	flag.BoolVar(&nosigname, "noname", false, "Do not print signal name")

	flag.Usage = Usage
	flag.Parse()

	if mask != "" {
		fmt.Println(DecodeSigmask(mask, nosigname))
		os.Exit(0)
	}

	args := flag.Args()

	if len(args) < 1 {
		fmt.Fprintf(os.Stderr, "missing process id or path to status file\n")
		flag.Usage()
		os.Exit(1)
	}
	if len(args) > 1 {
		fmt.Fprintf(os.Stderr, "unexpected arguments: %v", args[1:])
		flag.Usage()
		os.Exit(1)
	}

	path := args[0]

	file, err := os.Open(path)
	var err2 error
	if err != nil {
		file, err2 = os.Open(fmt.Sprintf("/proc/%s/status", path))
		if err2 != nil {
			fmt.Fprintf(os.Stderr, "error: %s\nerror: %s\n", err, err2)
			os.Exit(1)
		}
	}
	defer e.CloseOrExit(file)

	statusMap := ParseStatuses(file)

	printAll := true
	for _, mask := range sigmasks {
		if *mask.selected {
			printAll = false
			break
		}
	}
	for _, mask := range sigmasks {
		if printAll || *mask.selected {
			value := DecodeSigmask(statusMap[mask.name], nosigname)
			fmt.Printf("%s %s\n", mask.name, value)
		}
	}
}
