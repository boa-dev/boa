use crate::exec::Interpreter;
use crate::js::function::NativeFunctionData;
use crate::js::object::INSTANCE_PROTOTYPE;
use crate::js::value::{from_value, to_value, ResultValue, Value, ValueData};
use chrono::Local;
use gc::Gc;
use std::fmt::Write;
use std::iter::FromIterator;

/// Print a javascript value to the standard output stream
/// <https://console.spec.whatwg.org/#logger>
pub fn log(_: &Value, args: &[Value], _: &Interpreter) -> ResultValue {
    let args: Vec<String> = FromIterator::from_iter(args.iter().map(|x| {
        // Welcome to console.log! The output here is what the developer sees, so its best matching through value types and stringifying to the correct output
        // The input is a vector of Values, we generate a vector of strings then pass them to println!
        match *x.clone() {
            // We don't want to print private (compiler) or prototype properties
            ValueData::Object(ref v) => {
                // Create empty formatted string to start writing to
                // TODO: once constructor is set on objects, we can do specific output for Strings, Numbers etc
                let mut s = String::new();
                write!(s, "{{").unwrap();
                if let Some((last_key, _)) = v.borrow().properties.iter().last() {
                    for (key, val) in v.borrow().properties.iter() {
                        // Don't print prototype properties
                        if key == INSTANCE_PROTOTYPE {
                            continue;
                        }
                        write!(s, "{}: {}", key, val.value.clone()).unwrap();
                        if key != last_key {
                            write!(s, ", ").unwrap();
                        }
                    }
                }
                write!(s, "}}").unwrap();
                s
            }

            _ => from_value::<String>(x.clone()).unwrap(),
        }

        // from_value::<String>(x.clone()).unwrap()
    }));

    println!(
        "{}: {}",
        Local::now().format("%X").to_string(),
        args.join(" ")
    );
    Ok(Gc::new(ValueData::Undefined))
}
/// Print a javascript value to the standard error stream
pub fn error(_: &Value, args: &[Value], _: &Interpreter) -> ResultValue {
    let args: Vec<String> = FromIterator::from_iter(
        args.iter()
            .map(|x| from_value::<String>(x.clone()).unwrap()),
    );
    println!(
        "{}: {}",
        Local::now().format("%X").to_string(),
        args.join(" ")
    );
    Ok(Gc::new(ValueData::Undefined))
}
/// Create a new `console` object
pub fn _create(global: &Value) -> Value {
    let console = ValueData::new_obj(Some(global));
    console.set_field_slice("log", to_value(log as NativeFunctionData));
    console.set_field_slice("error", to_value(error as NativeFunctionData));
    console.set_field_slice("exception", to_value(error as NativeFunctionData));
    console
}
/// Initialise the global object with the `console` object
pub fn init(global: &Value) {
    global.set_field_slice("console", _create(global));
}
