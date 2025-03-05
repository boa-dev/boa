use boa_engine::{
    builtins::promise::PromiseState, js_string, object::builtins::JsPromise, Context, JsArgs,
    JsError, JsNativeError, JsResult, JsValue, NativeFunction,
};
use tokio;

// Simulate an API call that returns a Promise
async fn simulate_api_call(success: bool, delay_ms: u64) -> JsResult<JsValue> {
    // simulate network delay
    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

    if success {
        Ok(js_string!("API call successful!").into())
    } else {
        Err(JsError::from_native(
            JsNativeError::error().with_message("API call failed!"),
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let context = &mut Context::default();

    println!("1. Basic Promise Creation and Handling");
    // Create a promise that resolves after a delay
    let promise = JsPromise::new(
        |resolvers, context| {
            let result = js_string!("Hello from Promise!").into();
            resolvers
                .resolve
                .call(&JsValue::undefined(), &[result], context)?;
            Ok(JsValue::undefined())
        },
        context,
    );

    // Add success and error handlers
    let _promise = promise
        .then(
            Some(
                NativeFunction::from_fn_ptr(|_, args, _context| {
                    let value = args.get_or_undefined(0);
                    println!("Promise resolved with: {}", value.display());
                    Ok(value.clone())
                })
                .to_js_function(context.realm()),
            ),
            Some(
                NativeFunction::from_fn_ptr(|_, args, _context| {
                    let error = args.get_or_undefined(0);
                    println!("Promise rejected with: {}", error.display());
                    Err(JsError::from_opaque(error.clone()))
                })
                .to_js_function(context.realm()),
            ),
            context,
        )
        .finally(
            NativeFunction::from_fn_ptr(|_, _, _| {
                println!("Promise settled!");
                Ok(JsValue::undefined())
            })
            .to_js_function(context.realm()),
            context,
        );

    // Run the event loop to process promises
    drop(context.run_jobs());

    println!("\n2. Promise.all Example");
    // Create multiple promises
    let promises = vec![
        JsPromise::resolve(1, context),
        JsPromise::resolve(2, context),
        JsPromise::resolve(3, context),
    ];

    let all_promise = JsPromise::all(promises, context);
    drop(context.run_jobs());

    match all_promise.state() {
        PromiseState::Fulfilled(value) => {
            println!("All promises fulfilled with: {}", value.display());
        }
        PromiseState::Rejected(error) => {
            println!("One of the promises rejected with: {}", error.display());
        }
        PromiseState::Pending => {
            println!("Promises are still pending");
        }
    }

    println!("\n3. Promise.race Example");
    // Create promises that resolve at different times
    let (fast_promise, fast_resolvers) = JsPromise::new_pending(context);
    let (slow_promise, slow_resolvers) = JsPromise::new_pending(context);

    let race_promise = JsPromise::race([fast_promise, slow_promise], context);

    // Resolve promises in different order
    slow_resolvers
        .resolve
        .call(&JsValue::undefined(), &[js_string!("Slow").into()], context)?;
    fast_resolvers
        .resolve
        .call(&JsValue::undefined(), &[js_string!("Fast").into()], context)?;

    drop(context.run_jobs());

    if let PromiseState::Fulfilled(value) = race_promise.state() {
        println!("Race won by: {}", value.display());
    }

    println!("\n4. Converting Rust Future to Promise");
    // Create a promise from an async function
    let future_promise = JsPromise::from_future(simulate_api_call(true, 100), context);
    drop(context.run_jobs());

    match future_promise.state() {
        PromiseState::Fulfilled(value) => {
            println!("Future resolved with: {}", value.display());
        }
        PromiseState::Rejected(error) => {
            println!("Future rejected with: {}", error.display());
        }
        PromiseState::Pending => {
            println!("Future is still pending");
        }
    }

    println!("\n5. Promise.any Example");
    let promises = vec![
        JsPromise::reject(JsNativeError::error().with_message("Error 1"), context),
        JsPromise::resolve(js_string!("Success!"), context),
        JsPromise::reject(JsNativeError::error().with_message("Error 2"), context),
    ];

    let any_promise = JsPromise::any(promises, context);
    drop(context.run_jobs());

    match any_promise.state() {
        PromiseState::Fulfilled(value) => {
            println!("First fulfilled promise value: {}", value.display());
        }
        PromiseState::Rejected(error) => {
            println!("All promises rejected with: {}", error.display());
        }
        PromiseState::Pending => {
            println!("Promises are still pending");
        }
    }

    Ok(())
}
