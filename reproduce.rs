use boa_engine::{Context, Source};
use std::fs;

fn check_file(path: &str) {
    let Ok(file_content) = fs::read(path) else {
        return;
    };
    println!("Checking file: {path}");
    let mut context = Context::default();

    let _result = context.eval(Source::from_bytes(&file_content));
}

fn main() {
    check_file("oom_test.js");
}
