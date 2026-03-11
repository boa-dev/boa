use assert_cmd::Command;

#[test]
fn stdin_uncaught_error_exits_non_zero() {
    Command::cargo_bin("boa")
        .expect("boa binary should build")
        .write_stdin("throw Error('nooo')")
        .assert()
        .failure();
}

#[test]
fn expression_uncaught_error_exits_non_zero() {
    Command::cargo_bin("boa")
        .expect("boa binary should build")
        .args(["-e", "throw Error('nooo')"])
        .assert()
        .failure();
}
