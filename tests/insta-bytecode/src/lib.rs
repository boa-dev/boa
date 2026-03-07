use std::{
    fs::{self, File},
    path::PathBuf,
    process::{Command, Stdio},
};

pub const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub fn js_directory() -> PathBuf {
    PathBuf::from(MANIFEST_DIR).join("js")
}

pub fn target_directory() -> PathBuf {
    // NOTE: Github does weird things.
    let workspace_root = PathBuf::from(MANIFEST_DIR)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    if fs::exists(workspace_root.join("target")).is_ok() {
        workspace_root.join("target")
    } else {
        // We try to go up one more to deal with GitHub CI's boa/boa
        workspace_root.parent().unwrap().join("target")
    }
}

pub fn collect_file_trace(file_path: PathBuf) -> String {
    let file_path_msg = file_path.to_string_lossy().to_string();
    println!("Testing {}", file_path_msg);
    let boa_exe = target_directory()
        .join("debug/boa")
        .to_string_lossy()
        .to_string();
    println!("target: {:?}", std::env::var("CARGO_BIN_NAME"));
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

macro_rules! test_case {
    ($fn_name:ident, $js_filename:literal) => {
        #[test]
        fn $fn_name() {
            let output = collect_file_trace(js_directory().join("basicLoop.js"));
            insta::assert_snapshot!(output)
        }
    };
}

// Add test cases below
//
// Important note:
//
// The first arg is the function name / snapshot name
// The second arg is the js filename
//
test_case!(basic_loop, "basicLoop.js");
test_case!(double_loop_function, "doubleLoopFunction.js");
