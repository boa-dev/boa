#![allow(clippy::print_stdout)]

#[cfg(test)]
mod tests;

use crate::{
    builtins::{
        function::NativeFunctionData,
        object::InternalState,
        value::{from_value, to_value, FromValue, ResultValue, Value, ValueData},
    },
    exec::Interpreter,
};
use gc::Gc;
use std::{collections::HashMap, time::SystemTime};

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

fn get_arg_at_index<T: FromValue + Default>(args: &[Value], index: usize) -> Option<T> {
    args.get(index)
        .cloned()
        .map(|s| from_value::<T>(s).expect("Convert error"))
}

pub fn formatter(data: &[Value]) -> String {
    let target = get_arg_at_index::<String>(data, 0).unwrap_or_default();
    match data.len() {
        0 => String::new(),
        1 => target,
        _ => {
            let mut formatted = String::new();
            let mut arg_index = 1;
            let mut chars = target.chars();
            while let Some(c) = chars.next() {
                if c == '%' {
                    let fmt = chars.next().unwrap_or('%');
                    match fmt {
                        /* integer */
                        'd' | 'i' => {
                            let arg = get_arg_at_index::<i32>(data, arg_index).unwrap_or_default();
                            formatted.push_str(&format!("{}", arg));
                            arg_index += 1;
                        }
                        /* float */
                        'f' => {
                            let arg = get_arg_at_index::<f64>(data, arg_index).unwrap_or_default();
                            formatted.push_str(&format!("{number:.prec$}", number = arg, prec = 6));
                            arg_index += 1
                        }
                        /* object, FIXME: how to render this properly? */
                        'o' | 'O' => {
                            let arg = data.get(arg_index).cloned().unwrap_or_default();
                            formatted.push_str(&format!("{}", arg));
                            arg_index += 1
                        }
                        /* string */
                        's' => {
                            let arg =
                                get_arg_at_index::<String>(data, arg_index).unwrap_or_default();
                            formatted.push_str(&arg);
                            arg_index += 1
                        }
                        '%' => formatted.push('%'),
                        /* TODO: %c is not implemented */
                        c => {
                            formatted.push('%');
                            formatted.push(c);
                        }
                    }
                } else {
                    formatted.push(c);
                };
            }

            /* unformatted data */
            for rest in data.iter().skip(arg_index) {
                formatted.push_str(&format!(" {}", rest))
            }

            formatted
        }
    }
}

/// `console.assert(condition, ...data)`
///
/// Prints a JavaScript value to the standard error if first argument evaluates to `false` or there
/// were no arguments.
///
/// More information: <https://console.spec.whatwg.org/#assert>
pub fn assert(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let assertion = get_arg_at_index::<bool>(args, 0).unwrap_or_default();

    if !assertion {
        let mut args: Vec<Value> = args.iter().skip(1).cloned().collect();
        let message = "Assertion failed".to_string(); 
        if args.is_empty() {
            args.push(to_value::<String>(message));
        } else if !args[0].is_string() {
            args.insert(0, to_value::<String>(message));
        } else {
            let concat = format!("{}: {}", message, args[0]);
            args[0] = to_value::<String>(concat);
        }

        eprintln!("{}", formatter(&args[..]));
    }

    Ok(Gc::new(ValueData::Undefined))
}

/// `console.clear()`
///
/// Removes all groups and clears console if possible.
///
/// More information: <https://console.spec.whatwg.org/#clear>
pub fn clear(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_mut(|state: &mut ConsoleState| {
        state.groups.clear();
    });

    Ok(Gc::new(ValueData::Undefined))
}

/// `console.debug(...data)`
///
/// Prints a JavaScript values with "debug" logLevel.
///
/// More information: <https://console.spec.whatwg.org/#debug>
pub fn debug(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    println!("{}", formatter(args));
    Ok(Gc::new(ValueData::Undefined))
}

/// `console.error(...data)`
///
/// Prints a JavaScript values with "error" logLevel.
///
/// More information: <https://console.spec.whatwg.org/#error>
pub fn error(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    eprintln!("{}", formatter(args));
    Ok(Gc::new(ValueData::Undefined))
}

/// `console.info(...data)`
///
/// Prints a JavaScript values with "info" logLevel.
///
/// More information: <https://console.spec.whatwg.org/#info>
pub fn info(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    println!("{}", formatter(args));
    Ok(Gc::new(ValueData::Undefined))
}

/// `console.log(...data)`
///
/// Prints a JavaScript values with "log" logLevel.
///
/// More information: <https://console.spec.whatwg.org/#log>
pub fn log(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    // Welcome to console.log! The output here is what the developer sees, so its best matching through value types and stringifying to the correct output
    // The input is a vector of Values, we generate a vector of strings then
    // pass them to println!
    println!("{}", formatter(args));
    Ok(Gc::new(ValueData::Undefined))
}

/// `console.trace(...data)`
///
/// Prints a stack trace with "trace" logLevel, optionally labelled by data.
///
/// More information: <https://console.spec.whatwg.org/#trace>
pub fn trace(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    if !args.is_empty() {
        println!("{}", formatter(args));
    }

    /* TODO: get and print stack trace */
    println!("Not implemented: <stack trace>");
    Ok(Gc::new(ValueData::Undefined))
}

/// `console.warn(...data)`
///
/// Prints a JavaScript values with "warn" logLevel.
///
/// More information: <https://console.spec.whatwg.org/#warn>
pub fn warn(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    println!("{}", formatter(args));
    Ok(Gc::new(ValueData::Undefined))
}

/// `console.count(label)`
///
/// Prints number of times the function was called with that particular label.
///
/// More information: <https://console.spec.whatwg.org/#count>
pub fn count(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let label = get_arg_at_index::<String>(args, 0).unwrap_or_else(|| "default".to_string());

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
    let label = get_arg_at_index::<String>(args, 0).unwrap_or_else(|| "default".to_string());

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
    let label = get_arg_at_index::<String>(args, 0).unwrap_or_else(|| "default".to_string());

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
    let label = get_arg_at_index::<String>(args, 0).unwrap_or_else(|| "default".to_string());

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
    let label = get_arg_at_index::<String>(args, 0).unwrap_or_else(|| "default".to_string());

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

/// `console.group(...data)`
///
/// Adds new group with name from formatted data to stack.
///
/// More information: <https://console.spec.whatwg.org/#group>
pub fn group(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let group_label = formatter(args);

    this.with_internal_state_mut(|state: &mut ConsoleState| {
        println!("group: {}", &group_label);
        state.groups.push(group_label);
    });

    Ok(Gc::new(ValueData::Undefined))
}

/// `console.groupEnd(label)`
///
/// Removes the last group from the stack.
///
/// More information: <https://console.spec.whatwg.org/#groupend>
pub fn group_end(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_mut(|state: &mut ConsoleState| {
        state.groups.pop();
    });

    Ok(Gc::new(ValueData::Undefined))
}

/// Create a new `console` object
pub fn create_constructor(global: &Value) -> Value {
    let console = ValueData::new_obj(Some(global));
    console.set_field_slice("assert", to_value(assert as NativeFunctionData));
    console.set_field_slice("clear", to_value(clear as NativeFunctionData));
    console.set_field_slice("debug", to_value(debug as NativeFunctionData));
    console.set_field_slice("error", to_value(error as NativeFunctionData));
    console.set_field_slice("info", to_value(info as NativeFunctionData));
    console.set_field_slice("log", to_value(log as NativeFunctionData));
    console.set_field_slice("trace", to_value(trace as NativeFunctionData));
    console.set_field_slice("warn", to_value(warn as NativeFunctionData));
    console.set_field_slice("exception", to_value(error as NativeFunctionData));
    console.set_field_slice("count", to_value(count as NativeFunctionData));
    console.set_field_slice("countReset", to_value(count_reset as NativeFunctionData));
    console.set_field_slice("group", to_value(group as NativeFunctionData));
    console.set_field_slice("groupCollapsed", to_value(group as NativeFunctionData));
    console.set_field_slice("groupEnd", to_value(group_end as NativeFunctionData));
    console.set_field_slice("time", to_value(time as NativeFunctionData));
    console.set_field_slice("timeLog", to_value(time_log as NativeFunctionData));
    console.set_field_slice("timeEnd", to_value(time_end as NativeFunctionData));
    console.set_internal_state(ConsoleState::new());
    console
}
