#![allow(clippy::print_stdout)]

#[cfg(test)]
mod tests;

use crate::{
    builtins::{
        function::NativeFunctionData,
        object::InternalState,
        value::{display_obj, from_value, to_value, FromValue, ResultValue, Value, ValueData},
    },
    exec::Interpreter,
};
use gc::Gc;
use std::{collections::HashMap, time::SystemTime};

#[derive(Debug, Default)]
pub struct ConsoleState {
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

#[derive(Debug)]
pub enum LogMessage {
    Log(String),
    Info(String),
    Warn(String),
    Error(String),
}

fn get_arg_at_index<T: FromValue + Default>(args: &[Value], index: usize) -> Option<T> {
    args.get(index)
        .cloned()
        .map(|s| from_value::<T>(s).expect("Convert error"))
}

pub fn logger(msg: LogMessage, console_state: &ConsoleState) {
    let indent = 2 * console_state.groups.len();

    match msg {
        LogMessage::Error(msg) => {
            eprint!("{}", " ".repeat(indent));
            eprintln!("{}", msg);
        }
        LogMessage::Log(msg) | LogMessage::Info(msg) | LogMessage::Warn(msg) => {
            print!("{}", " ".repeat(indent));
            println!("{}", msg);
        }
    }
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
pub fn assert(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
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

        this.with_internal_state_ref(|state| {
            logger(LogMessage::Error(formatter(&args[..])), state)
        });
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
pub fn debug(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|state| logger(LogMessage::Log(formatter(&args[..])), state));
    Ok(Gc::new(ValueData::Undefined))
}

/// `console.error(...data)`
///
/// Prints a JavaScript values with "error" logLevel.
///
/// More information: <https://console.spec.whatwg.org/#error>
pub fn error(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|state| logger(LogMessage::Error(formatter(&args[..])), state));
    Ok(Gc::new(ValueData::Undefined))
}

/// `console.info(...data)`
///
/// Prints a JavaScript values with "info" logLevel.
///
/// More information: <https://console.spec.whatwg.org/#info>
pub fn info(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|state| logger(LogMessage::Info(formatter(&args[..])), state));
    Ok(Gc::new(ValueData::Undefined))
}

/// `console.log(...data)`
///
/// Prints a JavaScript values with "log" logLevel.
///
/// More information: <https://console.spec.whatwg.org/#log>
pub fn log(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|state| logger(LogMessage::Log(formatter(&args[..])), state));
    Ok(Gc::new(ValueData::Undefined))
}

/// `console.trace(...data)`
///
/// Prints a stack trace with "trace" logLevel, optionally labelled by data.
///
/// More information: <https://console.spec.whatwg.org/#trace>
pub fn trace(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    if !args.is_empty() {
        this.with_internal_state_ref(|state| logger(LogMessage::Log(formatter(&args[..])), state));

        /* TODO: get and print stack trace */
        this.with_internal_state_ref(|state| {
            logger(
                LogMessage::Log("Not implemented: <stack trace>".to_string()),
                state,
            )
        });
    }

    Ok(Gc::new(ValueData::Undefined))
}

/// `console.warn(...data)`
///
/// Prints a JavaScript values with "warn" logLevel.
///
/// More information: <https://console.spec.whatwg.org/#warn>
pub fn warn(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|state| logger(LogMessage::Warn(formatter(&args[..])), state));
    Ok(Gc::new(ValueData::Undefined))
}

/// `console.count(label)`
///
/// Prints number of times the function was called with that particular label.
///
/// More information: <https://console.spec.whatwg.org/#count>
pub fn count(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let label = get_arg_at_index::<String>(args, 0).unwrap_or_else(|| "default".to_string());

    this.with_internal_state_mut(|state: &mut ConsoleState| {
        let msg = format!("count {}:", &label);
        let c = state.count_map.entry(label).or_insert(0);
        *c += 1;

        logger(LogMessage::Info(format!("{} {}", msg, c)), state);
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

        logger(LogMessage::Warn(format!("countReset {}", label)), state);
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
            logger(
                LogMessage::Warn(format!("Timer '{}' already exist", label)),
                state,
            );
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
            let mut concat = format!("{}: {} ms", label, time - t);
            for msg in args.iter().skip(1) {
                concat = concat + " " + &msg.to_string();
            }
            logger(LogMessage::Log(concat), state);
        } else {
            logger(
                LogMessage::Warn(format!("Timer '{}' doesn't exist", label)),
                state,
            );
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
            logger(
                LogMessage::Info(format!("{}: {} ms - timer removed", label, time - t)),
                state,
            );
        } else {
            logger(
                LogMessage::Warn(format!("Timer '{}' doesn't exist", label)),
                state,
            );
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
        logger(LogMessage::Info(format!("group: {}", &group_label)), state);
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

/// `console.dir(item, options)`
///
/// Prints info about item
///
/// More information: <https://console.spec.whatwg.org/#dir>
pub fn dir(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_mut(|state: &mut ConsoleState| {
        logger(
            LogMessage::Info(display_obj(
                args.get(0).unwrap_or(&Gc::new(ValueData::Undefined)),
                true,
            )),
            state,
        );
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
    console.set_field_slice("dir", to_value(dir as NativeFunctionData));
    console.set_field_slice("dirxml", to_value(dir as NativeFunctionData));
    console.set_internal_state(ConsoleState::new());
    console
}
