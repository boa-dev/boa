use std::fs::read_to_string;

pub fn main() {
    let buffer = read_to_string("test.js").unwrap();
    println!("{}", buffer);
}