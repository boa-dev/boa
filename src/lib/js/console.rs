use gc::Gc;
use js::value::{from_value, to_value, ResultValue, Value, ValueData};
use std::iter::FromIterator;
use time::{now, strftime};
/// Print a javascript value to the standard output stream
pub fn log(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    let args: Vec<String> =
        FromIterator::from_iter(args.iter().map(|x| from_value::<String>(*x).unwrap()));
    println!("{}: {}", strftime("%X", &now()).unwrap(), args.join(" "));
    Ok(Gc::new(ValueData::Undefined))
}
/// Print a javascript value to the standard error stream
pub fn error(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    let args: Vec<String> = FromIterator::from_iter(
        args.iter()
            .map(|x| from_value::<String>(x.clone()).unwrap()),
    );
    eprintln!("{}: {}", strftime("%X", &now()).unwrap(), args.join(" "));
    Ok(Gc::new(ValueData::Undefined))
}
/// Create a new `console` object
pub fn _create(global: Value) -> Value {
    let console = ValueData::new_obj(Some(global));
    console.set_field_slice("log", to_value(log));
    console.set_field_slice("error", to_value(error));
    console.set_field_slice("exception", to_value(error));
    console
}
/// Initialise the global object with the `console` object
pub fn init(global: Value) {
    global.set_field_slice("console", _create(global.clone()));
}
