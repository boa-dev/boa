use boa_engine::{Context, JsValue, Source};

fn main() {
    // Create the JavaScript context.
    let mut context = Context::default();

    // -----------------------------------------
    //  Loop Iteration Limit
    // -----------------------------------------

    // Set the context's runtime limit on loops to 10 iterations.
    context.runtime_limits_mut().set_loop_iteration_limit(10);

    // The code below iterates 5 times, so no error is thrown.
    let result = context.eval(Source::from_bytes(
        r"
            for (let i = 0; i < 5; ++i) { }
        ",
    ));
    assert!(result.is_ok());

    // Here we exceed the limit by 1 iteration and a `RuntimeLimit` error is thrown.
    //
    // This error cannot be caught in JavaScript it propagates to rust caller.
    let result = context.eval(Source::from_bytes(
        r"
            try {
                for (let i = 0; i < 11; ++i) { }
            } catch (e) {

            }
        ",
    ));
    assert!(result.is_err());

    // Preventing an infinity loops
    let result = context.eval(Source::from_bytes(
        r"
            while (true) { }
        ",
    ));
    assert!(result.is_err());

    // The limit applies to all types of loops.
    let result = context.eval(Source::from_bytes(
        r"
            for (let e of [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]) { }
        ",
    ));
    assert!(result.is_err());

    // -----------------------------------------
    //  Recursion Limit
    // -----------------------------------------

    // Create and register `factorial` function.
    let result = context.eval(Source::from_bytes(
        r"
            function factorial(n) {
                if (n == 0) {
                    return 1;
                }

                return n * factorial(n - 1);
            }
        ",
    ));
    assert!(result.is_ok());

    // Run function before setting the limit and assert that it works.
    let result = context.eval(Source::from_bytes("factorial(11)"));
    assert_eq!(result, Ok(JsValue::new(39_916_800)));

    // Setting runtime limit for recustion to 10.
    context.runtime_limits_mut().set_recursion_limit(10);

    // Run without exceeding recursion limit and assert that it works.
    let result = context.eval(Source::from_bytes("factorial(8)"));
    assert_eq!(result, Ok(JsValue::new(40_320)));

    // Run exceeding limit by 1 and assert that it fails.
    let result = context.eval(Source::from_bytes("factorial(11)"));
    assert!(result.is_err());
}
