use std::{error::Error, path::Path, rc::Rc};

use boa_engine::{
    Context, JsError, JsNativeError, JsValue, Module, NativeFunction,
    builtins::promise::PromiseState, js_string, module::SimpleModuleLoader,
};
use boa_parser::Source;

// This example demonstrates how to use Boa's module API
fn main() -> Result<(), Box<dyn Error>> {
    // A simple module that we want to compile from Rust code.
    const MODULE_SRC: &str = r#"
        import { pyth } from "./trig.mjs";
        import * as ops from "./operations.mjs";

        export let result = pyth(3, 4);
        export function mix(a, b) {
            return ops.sum(ops.mult(a, ops.sub(b, a)), 10);
        }
    "#;

    // This can be overridden with any custom implementation of `ModuleLoader`.
    let loader = Rc::new(SimpleModuleLoader::new("./scripts/modules")?);

    // Just need to cast to a `ModuleLoader` before passing it to the builder.
    let context = &Context::builder().module_loader(loader.clone()).build()?;
    let source = Source::from_reader(MODULE_SRC.as_bytes(), Some(Path::new("./main.mjs")));

    // Can also pass a `Some(realm)` if you need to execute the module in another realm.
    let module = Module::parse(source, None, context)?;

    // Don't forget to insert the parsed module into the loader itself, since the root module
    // is not automatically inserted by the `ModuleLoader::load_imported_module` impl.
    //
    // Simulate as if the "fake" module is located in the modules root, just to ensure that
    // the loader won't double load in case someone tries to import "./main.mjs".
    loader.insert(
        Path::new("./scripts/modules")
            .canonicalize()?
            .join("main.mjs"),
        module.clone(),
    );

    // The lifecycle of the module is tracked using promises which can be a bit cumbersome to use.
    // If you just want to directly execute a module, you can use the `Module::load_link_evaluate`
    // method to skip all the boilerplate.
    // This does the full version for demonstration purposes.
    //
    // parse -> load -> link -> evaluate
    let promise_result = module
        // Initial load that recursively loads the module's dependencies.
        // This returns a `JsPromise` that will be resolved when loading finishes,
        // which allows async loads and async fetches.
        .load(context)
        .then(
            Some(
                NativeFunction::from_copy_closure_with_captures(
                    |_, _, module, context| {
                        // After loading, link all modules by resolving the imports
                        // and exports on the full module graph, initializing module
                        // environments. This returns a plain `Err` since all modules
                        // must link at the same time.
                        module.link(context)?;
                        Ok(JsValue::undefined())
                    },
                    module.clone(),
                )
                .to_js_function(context.realm()),
            ),
            None,
            context,
        )
        .then(
            Some(
                NativeFunction::from_copy_closure_with_captures(
                    // Finally, evaluate the root module.
                    // This returns a `JsPromise` since a module could have
                    // top-level await statements, which defers module execution to the
                    // job queue.
                    |_, _, module, context| Ok(module.evaluate(context)?.into()),
                    module.clone(),
                )
                .to_js_function(context.realm()),
            ),
            None,
            context,
        );

    // Very important to push forward the job queue after queueing promises.
    context.run_jobs()?;

    // Checking if the final promise didn't return an error.
    match promise_result.state() {
        PromiseState::Pending => return Err("module didn't execute!".into()),
        PromiseState::Fulfilled(v) => {
            assert_eq!(v, JsValue::undefined());
        }
        PromiseState::Rejected(err) => {
            return Err(JsError::from_opaque(err).try_native(context)?.into());
        }
    }

    // We can access the full namespace of the module with all its exports.
    let namespace = module.namespace(context);
    let result = namespace.get(js_string!("result"), context)?;

    println!("result = {}", result.display());

    assert_eq!(
        namespace.get(js_string!("result"), context)?,
        JsValue::from(5)
    );

    let mix = namespace
        .get(js_string!("mix"), context)?
        .as_callable()
        .ok_or_else(|| JsNativeError::typ().with_message("mix export wasn't a function!"))?;
    let result = mix.call(&JsValue::undefined(), &[5.into(), 10.into()], context)?;

    println!("mix(5, 10) = {}", result.display());

    assert_eq!(result, 35.into());

    Ok(())
}
