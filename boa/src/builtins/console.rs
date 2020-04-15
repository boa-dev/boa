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

/// `console.assert(condition, ...data)`
///
/// Prints a JavaScript value to the standard error if first argument evaluates to `false` or there
/// were no arguments.
///
/// More information: <https://console.spec.whatwg.org/#assert>
pub fn assert(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let assertion = args
        .get(0)
        .cloned()
        .map(|val| from_value::<bool>(val).expect("Could not convert to bool."))
        .unwrap_or_default();

    if !assertion {
        eprint!("Assertion failed:");
        for message in args.iter().skip(1) {
            eprint!(" {}", message);
        }
        eprintln!();
    }

    Ok(Gc::new(ValueData::Undefined))
}

/// `console.count(label)`
///
/// Prints number of times the function was called with that particular label.
///
/// More information: <https://console.spec.whatwg.org/#count>
pub fn count(v: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let label = args
        .get(0)
        .cloned()
        .map(|l| from_value::<String>(l).expect("Could not convert to string."))
        .unwrap_or_else(|| "default".to_string());

    let count_map = v.get_internal_slot("countMap");

    let c = match count_map.get_field_slice(&label) {
        v if v.is_undefined() => 1,
        v => from_value::<i32>(v).expect("Could not convert to i32.") + 1,
    };

    count_map.set_field_slice(&label, to_value(c));

    println!("count {}: {}", label, c);

    Ok(Gc::new(ValueData::Undefined))
}

/// `console.countReset(label)`
///
/// Resets the counter for label.
///
/// More information: <https://console.spec.whatwg.org/#countreset>
pub fn count_reset(v: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let label = args
        .get(0)
        .cloned()
        .map(|l| from_value::<String>(l).expect("Could not convert to string."))
        .unwrap_or_else(|| "default".to_string());

    let count_map = v.get_internal_slot("countMap");

    if count_map.has_field(&label) {
        count_map.set_field_slice(&label, to_value(0));
    }

    println!("countReset {}", label);

    Ok(Gc::new(ValueData::Undefined))
}
/// Create a new `console` object
pub fn create_constructor(global: &Value) -> Value {
    let console = ValueData::new_obj(Some(global));
    console.set_field_slice("log", to_value(log as NativeFunctionData));
    console.set_field_slice("error", to_value(error as NativeFunctionData));
    console.set_field_slice("exception", to_value(error as NativeFunctionData));
    console.set_field_slice("assert", to_value(assert as NativeFunctionData));
    console.set_field_slice("count", to_value(count as NativeFunctionData));
    console.set_field_slice("countReset", to_value(count_reset as NativeFunctionData));
    console.set_internal_slot("countMap", to_value(ValueData::new_obj(Some(global))));
    console
}
