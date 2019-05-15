extern crate boa;
use boa::exec;
use std::fs::read_to_string;

pub fn main() {
    let buffer = read_to_string("tests/js/test.js").unwrap();
    exec(buffer);
}
