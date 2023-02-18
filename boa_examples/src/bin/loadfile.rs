// This example shows how to load, parse and execute JS code from a source file
// (./scripts/helloworld.js)

use std::path::Path;

use boa_engine::{Context, Runtime, Source};

fn main() {
    let js_file_path = "./scripts/helloworld.js";

    match Source::from_filepath(Path::new(js_file_path)) {
        Ok(src) => {
            // Instantiate the execution context
            let runtime = &Runtime::default();
            let context = &mut Context::builder(runtime).build().unwrap();
            // Parse the source code
            match context.eval_script(src) {
                Ok(res) => {
                    println!(
                        "{}",
                        res.to_string(context).unwrap().to_std_string_escaped()
                    );
                }
                Err(e) => {
                    // Pretty print the error
                    eprintln!("Uncaught {e}");
                }
            };
        }
        Err(msg) => eprintln!("Error: {msg}"),
    }
}
