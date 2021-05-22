use boa::{exec::Executable, parse, Context};

pub fn run() {
    let js_code = "console.log('Hello World from a JS code string!')";

    // Instantiate the execution context
    let mut context = Context::new();

    // Parse the source code
    let expr = match parse(js_code, false) {
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
