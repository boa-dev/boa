use std::fs::read_to_string;

use boa_engine::Context;

pub fn run() {
    let js_file_path = "./scripts/helloworld.js";

    match read_to_string(js_file_path) {
        Ok(src) => {
            // Instantiate the execution context
            let mut context = Context::default();
            // Parse the source code
            match context.eval(src) {
                Ok(res) => {
                    println!("{}", res.to_string(&mut context).unwrap());
                }
                Err(e) => {
                    // Pretty print the error
                    eprintln!("Uncaught {}", e.display());
                }
            };
        }
        Err(msg) => eprintln!("Error: {}", msg),
    }
}
