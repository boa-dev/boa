#![allow(missing_docs)]

use std::io::Write;
use std::process::{Command, Stdio};

fn boa_bin() -> &'static str {
    env!("CARGO_BIN_EXE_boa")
}

#[test]
fn stdin_uncaught_error_exits_non_zero() {
    let mut child = Command::new(boa_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("boa binary should build");

    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(b"throw Error('nooo')")
        .expect("stdin write should succeed");

    let status = child.wait().expect("boa should exit");
    assert!(!status.success(), "expected non-zero exit for uncaught stdin error");
}

#[test]
fn expression_uncaught_error_exits_non_zero() {
    let status = Command::new(boa_bin())
        .args(["-e", "throw Error('nooo')"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("boa should run");

    assert!(!status.success(), "expected non-zero exit for uncaught -e error");
}
