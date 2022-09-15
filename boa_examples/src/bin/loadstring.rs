// This example loads, parses and executes a JS code string

use boa_engine::Context;

fn main() {
    let js_code = "console.log('Hello World from a JS code string!')";

    // Instantiate the execution context
    let mut context = Context::default();

    // Parse the source code
    match context.eval(js_code) {
        Ok(res) => {
            println!(
                "{}",
                res.to_string(&mut context).unwrap().to_std_string_escaped()
            );
        }
        Err(e) => {
            // Pretty print the error
            eprintln!("Uncaught {e}");
        }
    };
}
