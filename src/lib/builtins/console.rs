use crate::builtins::function::NativeFunctionData;
use crate::builtins::object::{ObjectKind, INSTANCE_PROTOTYPE};
use crate::builtins::value::{from_value, to_value, ResultValue, Value, ValueData};
use crate::exec::Interpreter;
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
                            .expect("Cannot get primitive value from String")
                            .clone(),
                    )
                    .expect("Cannot clone primitive value from String");
                    write!(s, "{}", str_val).unwrap();
                }
                ObjectKind::Boolean => {
                    let bool_data = v.borrow().get_internal_slot("BooleanData").to_string();
                    write!(s, "Boolean {{ {} }}", bool_data).unwrap();
                }
                ObjectKind::Array => {
                    write!(s, "[").unwrap();
                    let len: i32 = from_value(
                        v.borrow()
                            .properties
                            .get("length")
                            .unwrap()
                            .value
                            .clone()
                            .expect("Could not borrow value")
                            .clone(),
                    )
                    .expect("Could not convert JS value to i32");
                    for i in 0..len {
                        // Introduce recursive call to stringify any objects
                        // which are part of the Array
                        let arr_str = log_string_from(
                            v.borrow()
                                .properties
                                .get(&i.to_string())
                                .unwrap()
                                .value
                                .clone()
                                .expect("Could not borrow value")
                                .clone(),
                        );
                        write!(s, "{}", arr_str).unwrap();
                        if i != len.wrapping_sub(1) {
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
                            write!(
                                s,
                                "{}: {}",
                                key,
                                log_string_from(
                                    val.value.clone().expect("Could not read value").clone()
                                )
                            )
                            .unwrap();
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
        ValueData::Symbol(ref sym) => {
            let desc: Value = sym.borrow().get_internal_slot("Description");
            match *desc {
                ValueData::String(ref st) => format!("Symbol(\"{}\")", st.to_string()),
                _ => format!("Symbol()"),
            }
        }

        _ => from_value::<String>(x.clone()).expect("Could not convert value to String"),
    }
}

/// Print a javascript value to the standard output stream
/// <https://console.spec.whatwg.org/#logger>
pub fn log(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    // Welcome to console.log! The output here is what the developer sees, so its best matching through value types and stringifying to the correct output
    // The input is a vector of Values, we generate a vector of strings then
    // pass them to println!
    let args: Vec<String> =
        FromIterator::from_iter(args.iter().map(|x| log_string_from(x.clone())));

    println!("{}", args.join(" "));
    Ok(Gc::new(ValueData::Undefined))
}
/// Print a javascript value to the standard error stream
pub fn error(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let args: Vec<String> = FromIterator::from_iter(
        args.iter()
            .map(|x| from_value::<String>(x.clone()).expect("Could not convert value to String")),
    );
    println!("{}", args.join(" "));
    Ok(Gc::new(ValueData::Undefined))
}

/// Create a new `console` object
pub fn create_constructor(global: &Value) -> Value {
    let console = ValueData::new_obj(Some(global));
    console.set_field_slice("log", to_value(log as NativeFunctionData));
    console.set_field_slice("error", to_value(error as NativeFunctionData));
    console.set_field_slice("exception", to_value(error as NativeFunctionData));
    console
}
