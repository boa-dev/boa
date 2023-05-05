use boa_engine::{Context, Source};

fn main() {
    // Create the JavaScript context.
    let mut context = Context::default();

    // Set the context's runtime limit on loops to 10 iterations.
    context.runtime_limits_mut().set_loop_iteration_limit(10);

    // The code below iterates 5 times, so no error is thrown.
    let result = context.eval_script(Source::from_bytes(
        r"
            for (let i = 0; i < 5; ++i) { }
        ",
    ));
    result.expect("no error should be thrown");

    // Here we exceed the limit by 1 iteration and a `RuntimeLimit` error is thrown.
    //
    // This error cannot be caught in JavaScript it propagates to rust caller.
    let result = context.eval_script(Source::from_bytes(
        r"
            try {
                for (let i = 0; i < 11; ++i) { }
            } catch (e) {

            }
        ",
    ));
    result.expect_err("should have throw an error");

    // Preventing an infinity loops
    let result = context.eval_script(Source::from_bytes(
        r"
            while (true) { }
        ",
    ));
    result.expect_err("should have throw an error");

    // The limit applies to all types of loops.
    let result = context.eval_script(Source::from_bytes(
        r"
            for (let e of [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]) { }
        ",
    ));
    result.expect_err("should have throw an error");
}
