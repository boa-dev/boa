extern crate cbindgen;

use std::env;

// https://github.com/eqrion/cbindgen/blob/master/docs.md#buildrs

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::Builder::new()
      .with_crate(crate_dir)
      .with_language(cbindgen::Language::C)
      .generate()
      .expect("Unable to generate bindings")
      .write_to_file("boa.h");
}
