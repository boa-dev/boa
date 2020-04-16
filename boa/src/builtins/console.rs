#![allow(clippy::print_stdout)]

use crate::{
    builtins::{
        function::NativeFunctionData,
        object::InternalState,
        value::{from_value, log_string_from, to_value, ResultValue, Value, ValueData},
    },
    exec::Interpreter,
};
use gc::Gc;
use std::{collections::HashMap, iter::FromIterator, ops::Deref, time::SystemTime};

#[derive(Debug, Default)]
struct ConsoleState {
    count_map: HashMap<String, u32>,
    timer_map: HashMap<String, u128>,
    groups: Vec<String>,
}

impl ConsoleState {
    fn new() -> Self {
        ConsoleState {
            count_map: HashMap::new(),
            timer_map: HashMap::new(),
            groups: vec![],
        }
    }
}

impl InternalState for ConsoleState {}

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
pub fn count(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let label = args
        .get(0)
        .cloned()
        .map(|l| from_value::<String>(l).expect("Could not convert to string."))
        .unwrap_or_else(|| "default".to_string());

    print!("count {}: ", &label);

    this.with_internal_state_mut(|state: &mut ConsoleState| {
        let c = state.count_map.entry(label).or_insert(0);
        *c += 1;

        println!("{}", c);
    });

    Ok(Gc::new(ValueData::Undefined))
}

/// `console.countReset(label)`
///
/// Resets the counter for label.
///
/// More information: <https://console.spec.whatwg.org/#countreset>
pub fn count_reset(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let label = args
        .get(0)
        .cloned()
        .map(|l| from_value::<String>(l).expect("Could not convert to string."))
        .unwrap_or_else(|| "default".to_string());

    this.with_internal_state_mut(|state: &mut ConsoleState| {
        state.count_map.remove(&label);

        println!("countReset {}", label);
    });

    Ok(Gc::new(ValueData::Undefined))
}

/// Returns current system time in ms.
fn system_time_in_ms() -> u128 {
    let now = SystemTime::now();
    now.duration_since(SystemTime::UNIX_EPOCH)
        .expect("negative duration")
        .as_millis()
}

/// `console.time(label)`
///
/// Starts the timer for given label.
///
/// More information: <https://console.spec.whatwg.org/#time>
pub fn time(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let label = args
        .get(0)
        .cloned()
        .map(|l| from_value::<String>(l).expect("Could not convert to string."))
        .unwrap_or_else(|| "default".to_string());

    this.with_internal_state_mut(|state: &mut ConsoleState| {
        if state.timer_map.get(&label).is_some() {
            println!("Timer '{}' already exists", label)
        } else {
            let time = system_time_in_ms();
            state.timer_map.insert(label, time);
        }
    });

    Ok(Gc::new(ValueData::Undefined))
}

/// `console.timeLog(label, ...data)`
///
/// Prints elapsed time for timer with given label.
///
/// More information: <https://console.spec.whatwg.org/#timelog>
pub fn time_log(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let label = args
        .get(0)
        .cloned()
        .map(|l| from_value::<String>(l).expect("Could not convert to string."))
        .unwrap_or_else(|| "default".to_string());

    this.with_internal_state_mut(|state: &mut ConsoleState| {
        if let Some(t) = state.timer_map.get(&label) {
            let time = system_time_in_ms();
            print!("{}: {} ms", label, time - t);
            for message in args.iter().skip(1) {
                print!(" {}", message);
            }
            println!()
        } else {
            println!("Timer '{}' doesn't exist", label)
        }
    });

    Ok(Gc::new(ValueData::Undefined))
}

/// `console.timeEnd(label)`
///
/// Removes the timer with given label.
///
/// More information: <https://console.spec.whatwg.org/#timeend>
pub fn time_end(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let label = args
        .get(0)
        .cloned()
        .map(|l| from_value::<String>(l).expect("Could not convert to string."))
        .unwrap_or_else(|| "default".to_string());

    this.with_internal_state_mut(|state: &mut ConsoleState| {
        if let Some(t) = state.timer_map.remove(&label) {
            let time = system_time_in_ms();
            println!("{}: {} ms - timer removed", label, time - t)
        } else {
            println!("Timer '{}' doesn't exist", label)
        }
    });

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
    console.set_field_slice("time", to_value(time as NativeFunctionData));
    console.set_field_slice("timeLog", to_value(time_log as NativeFunctionData));
    console.set_field_slice("timeEnd", to_value(time_end as NativeFunctionData));
    console.set_internal_state(ConsoleState::new());
    console
}
