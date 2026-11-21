use std::io::Write;
use std::process::{Command, Output, Stdio};

const SHELL: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/shell.status");
const NO_SHDPND: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/no-shdpnd.status"
);

fn sigmask(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sigmask"))
        .args(args)
        .output()
        .expect("failed to run sigmask")
}

fn stdout(output: &Output) -> &str {
    std::str::from_utf8(&output.stdout).expect("stdout is not UTF-8")
}

fn stderr(output: &Output) -> &str {
    std::str::from_utf8(&output.stderr).expect("stderr is not UTF-8")
}

#[test]
fn prints_all_masks_from_status_file() {
    let output = sigmask(&[SHELL]);
    assert!(output.status.success());
    assert_eq!(
        stdout(&output),
        "SigPnd 0000000000000000 \n\
         ShdPnd 0000000000000000 \n\
         SigBlk 0000000000010000 SIGCHLD\n\
         SigIgn 0000000000384004 SIGQUIT,SIGTERM,SIGTSTP,SIGTTIN,SIGTTOU\n\
         SigCgt 000000004b813efb SIGHUP,SIGINT,SIGILL,SIGTRAP,SIGABRT,SIGBUS,SIGFPE,SIGUSR1,SIGSEGV,SIGUSR2,SIGPIPE,SIGALRM,SIGCHLD,SIGXCPU,SIGXFSZ,SIGVTALRM,SIGWINCH,SIGSYS\n"
    );
}

#[test]
fn prints_only_selected_masks_in_kernel_order() {
    let output = sigmask(&["--ignored", "-b", SHELL]);
    assert_eq!(
        stdout(&output),
        "SigBlk 0000000000010000 SIGCHLD\n\
         SigIgn 0000000000384004 SIGQUIT,SIGTERM,SIGTSTP,SIGTTIN,SIGTTOU\n"
    );
}

#[test]
fn noname_prints_numbers() {
    let output = sigmask(&["--noname", "--ignored", SHELL]);
    assert_eq!(stdout(&output), "SigIgn 0000000000384004 3,15,20,21,22\n");
}

#[test]
fn decodes_mask_argument() {
    let output = sigmask(&["--mask", "0000000000001000"]);
    assert!(output.status.success());
    assert_eq!(stdout(&output), "SIGPIPE\n");
}

#[test]
fn decodes_mask_argument_without_names() {
    let output = sigmask(&["-n", "-m", "0000000000001000"]);
    assert_eq!(stdout(&output), "13\n");
}

#[test]
fn rejects_invalid_mask() {
    let output = sigmask(&["--mask", "zz"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stderr(&output), "sigmask: error: invalid hex mask \"zz\"\n");
}

#[test]
fn reports_missing_field_without_partial_output() {
    let output = sigmask(&[NO_SHDPND]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("field ShdPnd not found"));
}

#[test]
fn reports_both_errors_for_unknown_target() {
    let output = sigmask(&["no-such-target"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stderr(&output).lines().count(), 2);
}

fn sigmask_stdin(args: &[&str], input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_sigmask"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to run sigmask");
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn reads_status_from_piped_stdin() {
    let input = std::fs::read(SHELL).unwrap();
    let output = sigmask_stdin(&["-b"], &input);
    assert!(output.status.success());
    assert_eq!(stdout(&output), "SigBlk 0000000000010000 SIGCHLD\n");
}

#[test]
fn dash_reads_stdin() {
    let input = std::fs::read(SHELL).unwrap();
    let output = sigmask_stdin(&["-n", "-i", "-"], &input);
    assert_eq!(stdout(&output), "SigIgn 0000000000384004 3,15,20,21,22\n");
}

#[test]
fn empty_stdin_reports_missing_field() {
    let output = sigmask_stdin(&[], b"");
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("stdin: field SigPnd not found"));
}

#[test]
fn closed_stdout_is_not_a_panic() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_sigmask"))
        .args(["--mask", "ffffffffffffffff"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert_eq!(stderr(&output), "");
}

#[test]
fn pid_target_ignores_same_named_file_in_cwd() {
    let dir = std::env::temp_dir().join(format!("sigmask-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let pid = std::process::id().to_string();
    std::fs::copy(NO_SHDPND, dir.join(&pid)).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_sigmask"))
        .args(["--shpending", &pid])
        .current_dir(&dir)
        .output()
        .unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(output.status.success(), "{}", stderr(&output));
}

#[test]
fn rejects_target_with_mask() {
    let output = sigmask(&["--mask", "1", SHELL]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn rejects_extra_arguments() {
    let output = sigmask(&[SHELL, SHELL]);
    assert_eq!(output.status.code(), Some(2));
}

#[cfg(target_os = "linux")]
#[test]
fn reads_own_process_by_pid() {
    let pid = std::process::id().to_string();
    let output = sigmask(&["--caught", &pid]);
    assert!(output.status.success());
    assert!(stdout(&output).starts_with("SigCgt "));
}
