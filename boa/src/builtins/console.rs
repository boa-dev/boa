#![allow(clippy::print_stdout)]

use crate::{
    builtins::{
        function::NativeFunctionData,
        value::{from_value, log_string_from, to_value, ResultValue, Value, ValueData},
    },
    exec::Interpreter,
};
use gc::Gc;
use std::{iter::FromIterator, ops::Deref};

/// Print a javascript value to the standard output stream
/// <https://console.spec.whatwg.org/#logger>
pub fn log(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    // Welcome to console.log! The output here is what the developer sees, so its best matching through value types and stringifying to the correct output
    // The input is a vector of Values, we generate a vector of strings then
    // pass them to println!
    let args: Vec<String> =
        FromIterator::from_iter(args.iter().map(|x| log_string_from(x.deref(), false)));

    println!("{}", args.join(" "));
    Ok(Gc::new(ValueData::Undefined))
}
/// Print a javascript value to the standard error stream
pub fn error(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let args: Vec<String> = FromIterator::from_iter(
        args.iter()
            .map(|x| from_value::<String>(x.clone()).expect("Could not convert value to String")),
    );
    eprintln!("{}", args.join(" "));
    Ok(Gc::new(ValueData::Undefined))
}

/// Print a javascript value to the standard error only if first argument evaluates to false or
/// there were no arguments
pub fn assert(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let raw_args: Vec<Value> = FromIterator::from_iter(args.iter().cloned().map(|x| x));
    let condition_false = !raw_args.is_empty()
        && !from_value::<bool>(raw_args[0].clone()).expect("Could not convert to bool.");

    if condition_false || raw_args.is_empty() {
        let msg = if raw_args.len() <= 1 {
            String::new()
        } else {
            let msgs: Vec<String> = FromIterator::from_iter(raw_args[1..].iter().map(|x| {
                from_value::<String>(x.clone()).expect("Could not convert value to string")
            }));
            msgs.join(" ")
        };
        eprintln!("Assertion failed: {}", msg);
    }

    Ok(Gc::new(ValueData::Undefined))
}

/// Create a new `console` object
pub fn create_constructor(global: &Value) -> Value {
    let console = ValueData::new_obj(Some(global));
    console.set_field_slice("log", to_value(log as NativeFunctionData));
    console.set_field_slice("error", to_value(error as NativeFunctionData));
    console.set_field_slice("exception", to_value(error as NativeFunctionData));
    console.set_field_slice("assert", to_value(assert as NativeFunctionData));
    console
}
