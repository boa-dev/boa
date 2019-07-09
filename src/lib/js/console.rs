use crate::js::function::NativeFunctionData;
use crate::js::object::{ObjectKind, INSTANCE_PROTOTYPE};
use crate::js::value::{from_value, to_value, ResultValue, Value, ValueData};
use chrono::Local;
use gc::Gc;
use std::fmt::Write;
use std::iter::FromIterator;

/// Create the String representation of the Javascript object or primitive for
/// printing
fn log_string_from(x: Value) -> String {
    match *x {
        // We don't want to print private (compiler) or prototype properties
        ValueData::Object(ref v) => {
            // Create empty formatted string to start writing to
            let mut s = String::new();
            // Can use the private "type" field of an Object to match on
            // which type of Object it represents for special printing
            match v.borrow().kind {
                ObjectKind::String => {
                    let str_val: String = from_value(
                        v.borrow()
                            .internal_slots
                            .get("PrimitiveValue")
                            .unwrap()
                            .clone(),
                    )
                    .unwrap();
                    write!(s, "{}", str_val).unwrap();
                }
                ObjectKind::Array => {
                    write!(s, "[").unwrap();
                    let len: i32 =
                        from_value(v.borrow().properties.get("length").unwrap().value.clone())
                            .unwrap();
                    for i in 0..len {
                        // Introduce recursive call to stringify any objects
                        // which are part of the Array
                        let arr_str = log_string_from(
                            v.borrow()
                                .properties
                                .get(&i.to_string())
                                .unwrap()
                                .value
                                .clone(),
                        );
                        write!(s, "{}", arr_str).unwrap();
                        if i != len - 1 {
                            write!(s, ", ").unwrap();
                        }
                    }
                    write!(s, "]").unwrap();
                }
                _ => {
                    write!(s, "{{").unwrap();
                    if let Some((last_key, _)) = v.borrow().properties.iter().last() {
                        for (key, val) in v.borrow().properties.iter() {
                            // Don't print prototype properties
                            if key == INSTANCE_PROTOTYPE {
                                continue;
                            }
                            // Introduce recursive call to stringify any objects
                            // which are keys of the object
                            write!(s, "{}: {}", key, log_string_from(val.value.clone())).unwrap();
                            if key != last_key {
                                write!(s, ", ").unwrap();
                            }
                        }
                    }
                    write!(s, "}}").unwrap();
                }
            }
            s
        }

        _ => from_value::<String>(x.clone()).unwrap(),
    }
}

/// Print a javascript value to the standard output stream
/// <https://console.spec.whatwg.org/#logger>
pub fn log(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    // Welcome to console.log! The output here is what the developer sees, so its best matching through value types and stringifying to the correct output
    // The input is a vector of Values, we generate a vector of strings then
    // pass them to println!
    let args: Vec<String> =
        FromIterator::from_iter(args.iter().map(|x| log_string_from(x.clone())));

    println!(
        "{}: {}",
        Local::now().format("%X").to_string(),
        args.join(" ")
    );
    Ok(Gc::new(ValueData::Undefined))
}
/// Print a javascript value to the standard error stream
pub fn error(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
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
