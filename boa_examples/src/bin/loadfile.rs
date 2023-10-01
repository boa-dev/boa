// This example shows how to load, parse and execute JS code from a source file
// (./scripts/helloworld.js)

use std::{error::Error, path::Path};

use boa_engine::{js_string, property::Attribute, Context, Source};
use boa_runtime::Console;

/// Adds the custom runtime to the context.
fn add_runtime(context: &mut Context<'_>) {
    // We first add the `console` object, to be able to call `console.log()`.
    let console = Console::init(context);
    context
        .register_global_property(js_string!(Console::NAME), console, Attribute::all())
        .expect("the console builtin shouldn't exist");
}

fn main() -> Result<(), Box<dyn Error>> {
    let js_file_path = "./scripts/helloworld.js";

    let source = Source::from_filepath(Path::new(js_file_path))?;

    // Instantiate the execution context
    let mut context = Context::default();
    // Add the runtime intrisics
    add_runtime(&mut context);
    // Parse the source code and print the result
    println!("{}", context.eval(source)?.display());

    Ok(())
}
