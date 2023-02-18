// This example loads, parses and executes a JS code string

use boa_engine::{Context, Runtime, Source};

fn main() {
    let js_code = "console.log('Hello World from a JS code string!')";

    // Instantiate the execution context
    let runtime = &Runtime::default();
    let context = &mut Context::builder(runtime).build().unwrap();

    // Parse the source code
    match context.eval_script(Source::from_bytes(js_code)) {
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
