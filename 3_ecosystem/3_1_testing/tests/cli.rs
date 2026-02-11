use std::io::Write;
use std::process::{Command, Output, Stdio};

fn run_with_input(args: &[&str], input: &str) -> Output {
    let exe = env!("CARGO_BIN_EXE_step_3_1");
    let mut child = Command::new(exe)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn step_3_1");

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .expect("failed to write stdin");
    }

    child
        .wait_with_output()
        .expect("failed to wait for step_3_1")
}

#[test]
fn wins_on_correct_guess() {
    let output = run_with_input(&["10"], "10\n");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Guess the number!"));
    assert!(stdout.contains("You win!"));
}

#[test]
fn handles_too_small_and_too_big() {
    let output = run_with_input(&["10"], "5\n15\n10\n");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Too small!"));
    assert!(stdout.contains("Too big!"));
    assert!(stdout.contains("You win!"));
}

#[test]
fn ignores_invalid_guess() {
    let output = run_with_input(&["10"], "nope\n10\n");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("You guessed: nope"));
    assert!(stdout.contains("You win!"));
}

#[test]
fn exits_on_missing_secret_number() {
    let output = run_with_input(&[], "");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No secret number is specified"));
}

#[test]
fn exits_on_invalid_secret_number() {
    let output = run_with_input(&["abc"], "");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Secret number is not a number"));
}
