//! Decode Linux process signal masks.
//!
//! `/proc/PID/status` reports a process's pending, blocked, ignored and caught
//! signals as hex bitmasks (`SigPnd`, `ShdPnd`, `SigBlk`, `SigIgn`, `SigCgt`).
//! Bit `n` (0-based) set means signal `n + 1` is in the set, see
//! `render_sigset_t` in the kernel's `fs/proc/array.c`.
//!
//! ```
//! use sigmask::{decode, format_signals};
//!
//! let signals = decode("0000000000384004")?;
//! assert_eq!(signals, [3, 15, 20, 21, 22]);
//! assert_eq!(format_signals(&signals, true), "SIGQUIT,SIGTERM,SIGTSTP,SIGTTIN,SIGTTOU");
//! # Ok::<(), sigmask::Error>(())
//! ```
//!
//! Reading a live process (Linux only):
//!
//! ```no_run
//! use sigmask::{MaskKind, ProcStatus};
//!
//! let status = ProcStatus::from_pid(std::process::id())?;
//! println!("{:?}", status.decode(MaskKind::Blocked)?);
//! # Ok::<(), sigmask::Error>(())
//! ```

mod signals;

use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub use signals::signal_name;

/// Errors returned by this crate.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Reading the status source failed.
    Io(io::Error),
    /// The mask is empty or contains a non-hex character.
    InvalidMask(String),
    /// The status data has no line for the requested field.
    MissingField(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => err.fmt(f),
            Error::InvalidMask(mask) => write!(f, "invalid hex mask {mask:?}"),
            Error::MissingField(field) => write!(f, "field {field} not found"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            Error::InvalidMask(_) | Error::MissingField(_) => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

/// The signal masks reported in `/proc/PID/status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaskKind {
    /// Signals pending for the thread (`SigPnd`).
    Pending,
    /// Signals pending for the process as a whole (`ShdPnd`).
    SharedPending,
    /// Blocked signals (`SigBlk`).
    Blocked,
    /// Ignored signals (`SigIgn`).
    Ignored,
    /// Signals with a handler installed (`SigCgt`).
    Caught,
}

impl MaskKind {
    /// All mask kinds, in the order the kernel prints them.
    pub const ALL: [MaskKind; 5] = [
        MaskKind::Pending,
        MaskKind::SharedPending,
        MaskKind::Blocked,
        MaskKind::Ignored,
        MaskKind::Caught,
    ];

    /// The field name used in `/proc/PID/status`.
    #[must_use]
    pub fn field(self) -> &'static str {
        match self {
            MaskKind::Pending => "SigPnd",
            MaskKind::SharedPending => "ShdPnd",
            MaskKind::Blocked => "SigBlk",
            MaskKind::Ignored => "SigIgn",
            MaskKind::Caught => "SigCgt",
        }
    }
}

/// Decodes a hex signal mask into the 1-based numbers of the signals it contains, in ascending order.
///
/// The mask may be any length; no `0x` prefix is accepted.
///
/// # Errors
///
/// Returns [`Error::InvalidMask`] if `mask` is empty or contains a non-hex character.
pub fn decode(mask: &str) -> Result<Vec<u32>, Error> {
    let invalid = || Error::InvalidMask(mask.to_owned());
    if mask.is_empty() {
        return Err(invalid());
    }
    let mut signals = Vec::new();
    for (nibble, c) in mask.chars().rev().enumerate() {
        let digit = c.to_digit(16).ok_or_else(invalid)?;
        for bit in 0..4 {
            if digit & (1 << bit) != 0 {
                let signo = u32::try_from(nibble * 4 + bit + 1).map_err(|_| invalid())?;
                signals.push(signo);
            }
        }
    }
    Ok(signals)
}

/// Joins signal numbers with commas, using names where known when `names` is true.
#[must_use]
pub fn format_signals(signals: &[u32], names: bool) -> String {
    signals
        .iter()
        .map(|&signo| match signal_name(signo) {
            Some(name) if names => name.to_owned(),
            _ => signo.to_string(),
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// Parses `Key: value` lines into a map with trimmed values.
///
/// Lines without a `:` are skipped; for repeated keys the last value wins.
/// Invalid UTF-8 (possible in process names) is replaced, not rejected.
///
/// # Errors
///
/// Returns [`Error::Io`] if reading fails.
pub fn parse_status<R: BufRead>(reader: R) -> Result<HashMap<String, String>, Error> {
    let mut fields = HashMap::new();
    for line in reader.split(b'\n') {
        let line = line?;
        let line = String::from_utf8_lossy(&line);
        if let Some((key, value)) = line.split_once(':') {
            fields.insert(key.to_owned(), value.trim().to_owned());
        }
    }
    Ok(fields)
}

/// The parsed contents of a `/proc/PID/status` file.
#[derive(Debug, Clone, Default)]
pub struct ProcStatus {
    fields: HashMap<String, String>,
}

impl ProcStatus {
    /// Parses status data from any reader.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if reading fails.
    pub fn from_reader<R: BufRead>(reader: R) -> Result<Self, Error> {
        Ok(ProcStatus {
            fields: parse_status(reader)?,
        })
    }

    /// Reads and parses a status file.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the file cannot be opened or read.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Self::from_reader(BufReader::new(File::open(path)?))
    }

    /// Reads and parses `/proc/<pid>/status`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the process does not exist or cannot be read.
    pub fn from_pid(pid: u32) -> Result<Self, Error> {
        Self::from_path(format!("/proc/{pid}/status"))
    }

    /// Returns the trimmed value of any status field, e.g. `"Name"`.
    pub fn get(&self, field: &str) -> Option<&str> {
        self.fields.get(field).map(String::as_str)
    }

    /// Returns the raw hex value of a signal mask.
    ///
    /// # Errors
    ///
    /// Returns [`Error::MissingField`] if the status data lacks the field.
    pub fn mask(&self, kind: MaskKind) -> Result<&str, Error> {
        self.get(kind.field())
            .ok_or(Error::MissingField(kind.field()))
    }

    /// Decodes a signal mask into signal numbers.
    ///
    /// # Errors
    ///
    /// Returns [`Error::MissingField`] if the field is absent, or
    /// [`Error::InvalidMask`] if its value is not valid hex.
    pub fn decode(&self, kind: MaskKind) -> Result<Vec<u32>, Error> {
        decode(self.mask(kind)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_zero_is_empty() {
        assert_eq!(decode("0000000000000000").unwrap(), Vec::<u32>::new());
    }

    #[test]
    fn decode_single_bit() {
        assert_eq!(decode("0000000000010000").unwrap(), [17]);
    }

    #[test]
    fn decode_all_bits() {
        assert_eq!(
            decode("ffffffffffffffff").unwrap(),
            (1..=64).collect::<Vec<_>>()
        );
    }

    #[test]
    fn decode_is_case_insensitive() {
        assert_eq!(decode("4B813EFB").unwrap(), decode("4b813efb").unwrap());
    }

    #[test]
    fn decode_beyond_64_bits() {
        assert_eq!(decode("10000000000000000").unwrap(), [65]);
    }

    #[test]
    fn decode_rejects_empty() {
        assert!(matches!(decode(""), Err(Error::InvalidMask(m)) if m.is_empty()));
    }

    #[test]
    fn decode_rejects_non_hex() {
        assert!(matches!(decode("00g0"), Err(Error::InvalidMask(m)) if m == "00g0"));
    }

    #[test]
    fn decode_rejects_prefix() {
        assert!(matches!(decode("0x10"), Err(Error::InvalidMask(_))));
    }

    #[test]
    fn signal_name_bounds() {
        assert_eq!(
            [
                signal_name(0),
                signal_name(1),
                signal_name(64),
                signal_name(65)
            ],
            [None, Some("SIGHUP"), Some("SIGRTMAX"), None]
        );
    }

    #[test]
    fn format_signals_with_names() {
        assert_eq!(format_signals(&[1, 15, 65], true), "SIGHUP,SIGTERM,65");
    }

    #[test]
    fn format_signals_without_names() {
        assert_eq!(format_signals(&[1, 15, 65], false), "1,15,65");
    }

    #[test]
    fn format_signals_empty() {
        assert_eq!(format_signals(&[], true), "");
    }

    #[test]
    fn parse_status_trims_values() {
        let fields = parse_status("SigBlk:\t0000000000010000\n".as_bytes()).unwrap();
        assert_eq!(fields["SigBlk"], "0000000000010000");
    }

    #[test]
    fn parse_status_keeps_colons_in_values() {
        let fields = parse_status("Name:\tfoo:bar\n".as_bytes()).unwrap();
        assert_eq!(fields["Name"], "foo:bar");
    }

    #[test]
    fn parse_status_skips_lines_without_colon() {
        let fields = parse_status("garbage\nSigIgn:\t0\n".as_bytes()).unwrap();
        assert_eq!(fields.len(), 1);
    }

    #[test]
    fn parse_status_accepts_invalid_utf8() {
        let fields = parse_status(&b"Name:\tf\xffo\nSigIgn:\t0\n"[..]).unwrap();
        assert_eq!(fields["SigIgn"], "0");
    }

    #[test]
    fn proc_status_missing_field() {
        let status = ProcStatus::from_reader("SigPnd:\t0\n".as_bytes()).unwrap();
        assert!(matches!(
            status.mask(MaskKind::SharedPending),
            Err(Error::MissingField("ShdPnd"))
        ));
    }

    #[test]
    fn proc_status_decode() {
        let status = ProcStatus::from_reader("SigIgn:\t0000000000001000\n".as_bytes()).unwrap();
        assert_eq!(status.decode(MaskKind::Ignored).unwrap(), [13]);
    }
}
