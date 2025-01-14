// This example implements a synthetic Rust module that is exposed to JS code.
// This mirrors the `modules.rs` example but uses synthetic modules instead.

use std::path::PathBuf;
use std::rc::Rc;
use std::{error::Error, path::Path};

use boa_engine::builtins::promise::PromiseState;
use boa_engine::module::{SimpleModuleLoader, SyntheticModuleInitializer};
use boa_engine::object::FunctionObjectBuilder;
use boa_engine::{
    js_string, Context, JsArgs, JsError, JsNativeError, JsValue, Module, NativeFunction, Source,
};

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

    // This can be overriden with any custom implementation of `ModuleLoader`.
    let loader = Rc::new(SimpleModuleLoader::new("./scripts/modules")?);

    // Just need to cast to a `ModuleLoader` before passing it to the builder.
    let context = &mut Context::builder().module_loader(loader.clone()).build()?;

    // Now, create the synthetic module and insert it into the loader.
    let operations = create_operations_module(context);
    loader.insert(
        PathBuf::from("./scripts/modules")
            .canonicalize()?
            .join("operations.mjs"),
        operations,
    );

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

    // This uses the utility function to load, link and evaluate a module without having to deal
    // with callbacks. For an example demonstrating the whole lifecycle of a module, see
    // `modules.rs`
    let promise_result = module.load_link_evaluate(context);

    // Very important to push forward the job queue after queueing promises.
    context.run_jobs()?;

    // Checking if the final promise didn't return an error.
    match promise_result.state() {
        PromiseState::Pending => return Err("module didn't execute!".into()),
        PromiseState::Fulfilled(v) => {
            assert_eq!(v, JsValue::undefined());
        }
        PromiseState::Rejected(err) => {
            return Err(JsError::from_opaque(err).try_native(context)?.into())
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
        .cloned()
        .ok_or_else(|| JsNativeError::typ().with_message("mix export wasn't a function!"))?;
    let result = mix.call(&JsValue::undefined(), &[5.into(), 10.into()], context)?;

    println!("mix(5, 10) = {}", result.display());

    assert_eq!(result, 35.into());

    Ok(())
}

// Creates the synthetic equivalent to the `./modules/operations.mjs` file.
fn create_operations_module(context: &mut Context) -> Module {
    // We first create the function objects that will be exported by the module. More
    // on that below.
    let sum = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(|_, args, ctx| {
            args.get_or_undefined(0).add(args.get_or_undefined(1), ctx)
        }),
    )
    .length(2)
    .name("sum")
    .build();
    let sub = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(|_, args, ctx| {
            args.get_or_undefined(0).sub(args.get_or_undefined(1), ctx)
        }),
    )
    .length(2)
    .name("sub")
    .build();
    let mult = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(|_, args, ctx| {
            args.get_or_undefined(0).mul(args.get_or_undefined(1), ctx)
        }),
    )
    .length(2)
    .name("mult")
    .build();
    let div = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(|_, args, ctx| {
            args.get_or_undefined(0).div(args.get_or_undefined(1), ctx)
        }),
    )
    .length(2)
    .name("div")
    .build();
    let sqrt = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(|_, args, ctx| {
            let a = args.get_or_undefined(0).to_number(ctx)?;
            Ok(JsValue::from(a.sqrt()))
        }),
    )
    .length(1)
    .name("sqrt")
    .build();

    Module::synthetic(
        // Make sure to list all exports beforehand.
        &[
            js_string!("sum"),
            js_string!("sub"),
            js_string!("mult"),
            js_string!("div"),
            js_string!("sqrt"),
        ],
        // The initializer is evaluated every time a module imports this synthetic module,
        // so we avoid creating duplicate objects by capturing and cloning them instead.
        SyntheticModuleInitializer::from_copy_closure_with_captures(
            |module, fns, _| {
                println!("Running initializer!");
                module.set_export(&js_string!("sum"), fns.0.clone().into())?;
                module.set_export(&js_string!("sub"), fns.1.clone().into())?;
                module.set_export(&js_string!("mult"), fns.2.clone().into())?;
                module.set_export(&js_string!("div"), fns.3.clone().into())?;
                module.set_export(&js_string!("sqrt"), fns.4.clone().into())?;
                Ok(())
            },
            (sum, sub, mult, div, sqrt),
        ),
        None,
        None,
        context,
    )
}
