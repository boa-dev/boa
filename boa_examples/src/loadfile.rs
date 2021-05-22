use std::fs::read_to_string;

use boa::{exec::Executable, parse, Context};

pub fn run() {
    let js_file_path = "./scripts/helloworld.js";

    match read_to_string(js_file_path) {
        Ok(src) => {
            // Instantiate the execution context
            let mut context = Context::new();

            // Parse the source code
            let expr = match parse(src, false) {
                Ok(res) => res,
                Err(e) => {
                    // Pretty print the error
                    eprintln!(
                        "Uncaught {}",
                        context
                            .throw_syntax_error(e.to_string())
                            .expect_err("interpreter.throw_syntax_error() did not return an error")
                            .display()
                    );

                    return;
                }
            };

            // Execute the JS code read from the source file
            match expr.run(&mut context) {
                Ok(v) => println!("{}", v.display()),
                Err(e) => eprintln!("Uncaught {}", e.display()),
            }
        }
        Err(msg) => eprintln!("Error: {}", msg),
    }
}
