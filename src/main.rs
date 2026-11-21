use std::io::{self, BufReader, IsTerminal, Write};
use std::process::ExitCode;

use clap::{CommandFactory, Parser};
use sigmask::{MaskKind, ProcStatus, decode, format_signals};

/// Decode and print process signal masks from /proc/PID/status.
#[derive(Parser)]
#[allow(clippy::struct_excessive_bools, clippy::doc_markdown)] // one bool per flag; help text is user-facing
#[command(version)]
struct Cli {
    /// Process id, or path to a proc status file ("-" or piped input reads stdin)
    #[arg(value_name = "PID|PATH", conflicts_with = "mask")]
    target: Option<String>,

    /// Show pending (SigPnd)
    #[arg(short, long, conflicts_with = "mask")]
    pending: bool,

    /// Show shared pending (ShdPnd)
    #[arg(short, long, conflicts_with = "mask")]
    shpending: bool,

    /// Show blocked (SigBlk)
    #[arg(short, long, conflicts_with = "mask")]
    blocked: bool,

    /// Show ignored (SigIgn)
    #[arg(short, long, conflicts_with = "mask")]
    ignored: bool,

    /// Show caught (SigCgt)
    #[arg(short, long, conflicts_with = "mask")]
    caught: bool,

    /// Decode a single hex mask instead of reading a status file
    #[arg(short, long, value_name = "MASK")]
    mask: Option<String>,

    /// Print signal numbers instead of names
    #[arg(short, long)]
    noname: bool,
}

impl Cli {
    fn selected(&self) -> Vec<MaskKind> {
        let flags = [
            (MaskKind::Pending, self.pending),
            (MaskKind::SharedPending, self.shpending),
            (MaskKind::Blocked, self.blocked),
            (MaskKind::Ignored, self.ignored),
            (MaskKind::Caught, self.caught),
        ];
        let chosen: Vec<_> = flags
            .iter()
            .filter(|(_, on)| *on)
            .map(|(k, _)| *k)
            .collect();
        if chosen.is_empty() {
            MaskKind::ALL.to_vec()
        } else {
            chosen
        }
    }
}

/// Loads status data from stdin, a PID, or a path.
///
/// `None` reads stdin when it is piped; `Some("-")` always reads stdin.
fn load(target: Option<&str>) -> Result<ProcStatus, String> {
    let target = match target {
        Some("-") => None,
        Some(t) => Some(t),
        None if io::stdin().is_terminal() => {
            Cli::command()
                .error(
                    clap::error::ErrorKind::MissingRequiredArgument,
                    "a PID|PATH or --mask is required (or pipe a status file to stdin)",
                )
                .exit();
        }
        None => None,
    };
    let Some(target) = target else {
        return ProcStatus::from_reader(BufReader::new(io::stdin().lock()))
            .map_err(|e| format!("stdin: {e}"));
    };
    let proc_path = format!("/proc/{target}/status");
    let from_proc = |path_err: Option<sigmask::Error>| {
        ProcStatus::from_path(&proc_path).map_err(|proc_err| match path_err {
            Some(e) => format!("{target}: {e}\n{proc_path}: {proc_err}"),
            None => format!("{proc_path}: {proc_err}"),
        })
    };
    // An all-digit target is a PID: don't let a same-named file in the cwd shadow it.
    if target.bytes().all(|b| b.is_ascii_digit()) {
        return from_proc(None).or_else(|proc_err| {
            ProcStatus::from_path(target)
                .map_err(|path_err| format!("{target}: {path_err}\n{proc_err}"))
        });
    }
    ProcStatus::from_path(target).or_else(|path_err| from_proc(Some(path_err)))
}

/// Prints to stdout; a closed pipe (e.g. `| head`) is not an error.
fn emit(text: &str) -> Result<(), String> {
    match writeln!(io::stdout().lock(), "{text}") {
        Err(e) if e.kind() != io::ErrorKind::BrokenPipe => Err(format!("write error: {e}")),
        _ => Ok(()),
    }
}

fn run(cli: &Cli) -> Result<(), String> {
    let names = !cli.noname;

    if let Some(mask) = &cli.mask {
        let signals = decode(mask).map_err(|e| e.to_string())?;
        return emit(&format_signals(&signals, names));
    }

    let status = load(cli.target.as_deref())?;
    let label = cli.target.as_deref().unwrap_or("stdin");

    let mut lines = Vec::new();
    for kind in cli.selected() {
        let raw = status.mask(kind).map_err(|e| format!("{label}: {e}"))?;
        let signals = decode(raw).map_err(|e| format!("{label}: {}: {e}", kind.field()))?;
        lines.push(format!(
            "{} {raw} {}",
            kind.field(),
            format_signals(&signals, names)
        ));
    }
    emit(&lines.join("\n"))
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            for line in msg.lines() {
                eprintln!("sigmask: error: {line}");
            }
            ExitCode::FAILURE
        }
    }
}
