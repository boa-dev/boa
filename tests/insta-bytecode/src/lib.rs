use std::{
    fs::File,
    path::PathBuf,
    process::{Command, Stdio},
};

pub const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub fn js_directory() -> PathBuf {
    PathBuf::from(MANIFEST_DIR).join("js")
}

pub fn target_diretory() -> PathBuf {
    PathBuf::from(MANIFEST_DIR)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target")
}

pub fn collect_file_trace(file_path: PathBuf) -> String {
    let file_path_msg = file_path.to_string_lossy().to_string();
    println!("Testing {}", file_path_msg);
    let boa_exe = target_diretory()
        .join("debug/boa")
        .to_string_lossy()
        .to_string();
    println!("Using boa: {boa_exe:?}");
    let result = Command::new(boa_exe)
        .args(["--trace"])
        .stdin(Stdio::from(File::open(file_path).unwrap()))
        .output()
        .unwrap();
    if result.status.success() {
        let full_trace = String::from_utf8_lossy(&result.stdout).to_string();
        let (bytecode, _trace) = full_trace
            .split_once("\n\n")
            .expect("trace block should have two line breaks");
        bytecode.to_owned()
    } else {
        let failure_msg = String::from_utf8_lossy(&result.stderr).to_string();
        panic!("boa failed: {}", failure_msg);
    }
}

#[test]
fn basic_loop() {
    let output = collect_file_trace(js_directory().join("basicLoop.js"));
    insta::with_settings!({filters => vec![
        (r"[0-9]+μs", "[time]")
    ]}, {
        insta::assert_snapshot!(output)
    })
}

#[test]
fn double_loop_function() {
    let output = collect_file_trace(js_directory().join("doubleLoopFunction.js"));
    insta::with_settings!({filters => vec![
        (r"[0-9]+μs", "[time]")
    ]}, {
        insta::assert_snapshot!(output)
    })
}
