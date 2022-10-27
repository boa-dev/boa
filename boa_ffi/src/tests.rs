use std::io::{Write};
use std::process::{Command, Stdio};
use std::str;

// https://stackoverflow.com/questions/49218599/write-to-child-process-stdin-in-rust

fn get_output(input: &str) -> String {
    let mut child = Command::new("test/boa_test")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let child_stdin = child.stdin.as_mut().unwrap();
    child_stdin.write_all(input.as_bytes()).unwrap();
    // Close stdin to finish and avoid indefinite blocking
    drop(child_stdin);
    
    let output = child.wait_with_output().unwrap();

    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn no_input() {
    assert_eq!(get_output(r#""#), r#"undefined"#);
}

#[test]
fn json_output() {
    assert_eq!(get_output(r#"JSON.stringify({a:7});"#), r#""{"a":7}""#);
}

#[test]
fn check_utf8() {
    // This is a little bit tricky. The first string includes the characters
    // \u20ac as literal characters. When processed by JSON it yields a
    // Unicode 20AC (EURO SIGN). Which is then compared to a Rust string with
    // an encoded \u{20ac}
    assert_eq!(get_output(r#"JSON.stringify({a:"100\u20ac"});"#), "\"{\"a\":\"100\u{20ac}\"}\"");
}
