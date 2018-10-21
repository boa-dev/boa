use js::function::Function;
use js::value::{from_value, ResultValue, Value};
use std::iter::FromIterator;
use time::{now, strftime};
/// Print a javascript value to the standard output stream
pub fn log(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    let args: Vec<String> = FromIterator::from_iter(
        args.iter()
            .map(|x| from_value::<String>(x.clone()).unwrap()),
    );
    println!("{}: {}", strftime("%X", &now()).unwrap(), args.join(" "));
    Ok(Value::undefined())
}
/// Print a javascript value to the standard error stream
pub fn error(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    let args: Vec<String> = FromIterator::from_iter(
        args.iter()
            .map(|x| from_value::<String>(x.clone()).unwrap()),
    );
    eprintln!("{}: {}", strftime("%X", &now()).unwrap(), args.join(" "));
    Ok(Value::undefined())
}
/// Create a new `console` object
pub fn _create(global: Value) -> Value {
    let console = Value::new_obj(Some(global));
    console.set_field_slice("log", Function::make(log, &["object"]));
    console.set_field_slice("error", Function::make(error, &["error"]));
    console.set_field_slice("exception", Function::make(error, &["error"]));
    console
}
/// Initialise the global object with the `console` object
pub fn init(global: Value) {
    global.set_field_slice("console", _create(global.clone()));
}
