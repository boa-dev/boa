//! This module implements the global `console` object.
//!
//! The `console` object can be accessed from any global object.
//!
//! The specifics of how it works varies from browser to browser, but there is a de facto set of features that are typically provided.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [WHATWG `console` specification][spec]
//!
//! [spec]: https://console.spec.whatwg.org/
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Console

#![allow(clippy::print_stdout)]

#[cfg(test)]
mod tests;

use crate::{
    builtins::{
        object::InternalState,
        value::{
            display_obj, from_value, to_value, undefined, FromValue, ResultValue, Value, ValueData,
        },
    },
    exec::Interpreter,
};
use rustc_hash::FxHashMap;
use std::time::SystemTime;

/// This is the internal console object state.
#[derive(Debug, Default)]
pub struct ConsoleState {
    count_map: FxHashMap<String, u32>,
    timer_map: FxHashMap<String, u128>,
    groups: Vec<String>,
}

impl InternalState for ConsoleState {}

/// This represents the different types of log messages.
#[derive(Debug)]
pub enum LogMessage {
    Log(String),
    Info(String),
    Warn(String),
    Error(String),
}

/// Helper function that returns the argument at a specified index.
fn get_arg_at_index<T: FromValue + Default>(args: &[Value], index: usize) -> Option<T> {
    args.get(index)
        .cloned()
        .map(|s| from_value::<T>(s).expect("Convert error"))
}

/// Helper function for logging messages.
pub fn logger(msg: LogMessage, console_state: &ConsoleState) {
    let indent = 2 * console_state.groups.len();

    match msg {
        LogMessage::Error(msg) => {
            eprintln!("{:>width$}", msg, width = indent);
        }
        LogMessage::Log(msg) | LogMessage::Info(msg) | LogMessage::Warn(msg) => {
            println!("{:>width$}", msg, width = indent);
        }
    }
}

/// This represents the `console` formatter.
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
/// More information:
///  - [MDN documentation][mdn]
///  - [WHATWG `console` specification][spec]
///
/// [spec]: https://console.spec.whatwg.org/#assert
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/console/assert
pub fn assert(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
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

    Ok(undefined())
}

/// `console.clear()`
///
/// Removes all groups and clears console if possible.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [WHATWG `console` specification][spec]
///
/// [spec]: https://console.spec.whatwg.org/#clear
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/console/clear
pub fn clear(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_mut(|state: &mut ConsoleState| {
        state.groups.clear();
    });

    Ok(undefined())
}

/// `console.debug(...data)`
///
/// Prints a JavaScript values with "debug" logLevel.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [WHATWG `console` specification][spec]
///
/// [spec]: https://console.spec.whatwg.org/#debug
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/console/debug
pub fn debug(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|state| logger(LogMessage::Log(formatter(&args[..])), state));
    Ok(undefined())
}

/// `console.error(...data)`
///
/// Prints a JavaScript values with "error" logLevel.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [WHATWG `console` specification][spec]
///
/// [spec]: https://console.spec.whatwg.org/#error
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/console/error
pub fn error(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|state| logger(LogMessage::Error(formatter(&args[..])), state));
    Ok(undefined())
}

/// `console.info(...data)`
///
/// Prints a JavaScript values with "info" logLevel.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [WHATWG `console` specification][spec]
///
/// [spec]: https://console.spec.whatwg.org/#info
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/console/info
pub fn info(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|state| logger(LogMessage::Info(formatter(&args[..])), state));
    Ok(undefined())
}

/// `console.log(...data)`
///
/// Prints a JavaScript values with "log" logLevel.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [WHATWG `console` specification][spec]
///
/// [spec]: https://console.spec.whatwg.org/#log
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/console/log
pub fn log(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|state| logger(LogMessage::Log(formatter(&args[..])), state));
    Ok(undefined())
}

/// `console.trace(...data)`
///
/// Prints a stack trace with "trace" logLevel, optionally labelled by data.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [WHATWG `console` specification][spec]
///
/// [spec]: https://console.spec.whatwg.org/#trace
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/console/trace
pub fn trace(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
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

    Ok(undefined())
}

/// `console.warn(...data)`
///
/// Prints a JavaScript values with "warn" logLevel.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [WHATWG `console` specification][spec]
///
/// [spec]: https://console.spec.whatwg.org/#warn
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/console/warn
pub fn warn(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref(|state| logger(LogMessage::Warn(formatter(&args[..])), state));
    Ok(undefined())
}

/// `console.count(label)`
///
/// Prints number of times the function was called with that particular label.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [WHATWG `console` specification][spec]
///
/// [spec]: https://console.spec.whatwg.org/#count
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/console/count
pub fn count(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let label = get_arg_at_index::<String>(args, 0).unwrap_or_else(|| "default".to_string());

    this.with_internal_state_mut(|state: &mut ConsoleState| {
        let msg = format!("count {}:", &label);
        let c = state.count_map.entry(label).or_insert(0);
        *c += 1;

        logger(LogMessage::Info(format!("{} {}", msg, c)), state);
    });

    Ok(undefined())
}

/// `console.countReset(label)`
///
/// Resets the counter for label.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [WHATWG `console` specification][spec]
///
/// [spec]: https://console.spec.whatwg.org/#countreset
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/countReset
pub fn count_reset(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let label = get_arg_at_index::<String>(args, 0).unwrap_or_else(|| "default".to_string());

    this.with_internal_state_mut(|state: &mut ConsoleState| {
        state.count_map.remove(&label);

        logger(LogMessage::Warn(format!("countReset {}", label)), state);
    });

    Ok(undefined())
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
/// More information:
///  - [MDN documentation][mdn]
///  - [WHATWG `console` specification][spec]
///
/// [spec]: https://console.spec.whatwg.org/#time
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/console/time
pub fn time(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
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

    Ok(undefined())
}

/// `console.timeLog(label, ...data)`
///
/// Prints elapsed time for timer with given label.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [WHATWG `console` specification][spec]
///
/// [spec]: https://console.spec.whatwg.org/#timelog
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/console/timeLog
pub fn time_log(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
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

    Ok(undefined())
}

/// `console.timeEnd(label)`
///
/// Removes the timer with given label.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [WHATWG `console` specification][spec]
///
/// [spec]: https://console.spec.whatwg.org/#timeend
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/console/timeEnd
pub fn time_end(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
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

    Ok(undefined())
}

/// `console.group(...data)`
///
/// Adds new group with name from formatted data to stack.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [WHATWG `console` specification][spec]
///
/// [spec]: https://console.spec.whatwg.org/#group
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/console/group
pub fn group(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let group_label = formatter(args);

    this.with_internal_state_mut(|state: &mut ConsoleState| {
        logger(LogMessage::Info(format!("group: {}", &group_label)), state);
        state.groups.push(group_label);
    });

    Ok(undefined())
}

/// `console.groupEnd(label)`
///
/// Removes the last group from the stack.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [WHATWG `console` specification][spec]
///
/// [spec]: https://console.spec.whatwg.org/#groupend
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/console/groupEnd
pub fn group_end(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_mut(|state: &mut ConsoleState| {
        state.groups.pop();
    });

    Ok(undefined())
}

/// `console.dir(item, options)`
///
/// Prints info about item
///
/// More information:
///  - [MDN documentation][mdn]
///  - [WHATWG `console` specification][spec]
///
/// [spec]: https://console.spec.whatwg.org/#dir
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/console/dir
pub fn dir(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    this.with_internal_state_mut(|state: &mut ConsoleState| {
        logger(
            LogMessage::Info(display_obj(args.get(0).unwrap_or(&undefined()), true)),
            state,
        );
    });

    Ok(undefined())
}

/// Create a new `console` object
pub fn create(global: &Value) -> Value {
    let console = ValueData::new_obj(Some(global));

    make_builtin_fn!(assert, named "assert", of console);
    make_builtin_fn!(clear, named "clear", of console);
    make_builtin_fn!(debug, named "debug", of console);
    make_builtin_fn!(error, named "error", of console);
    make_builtin_fn!(info, named "info", of console);
    make_builtin_fn!(log, named "log", of console);
    make_builtin_fn!(trace, named "trace", of console);
    make_builtin_fn!(warn, named "warn", of console);
    make_builtin_fn!(error, named "exception", of console);
    make_builtin_fn!(count, named "count", of console);
    make_builtin_fn!(count_reset, named "countReset", of console);
    make_builtin_fn!(group, named "group", of console);
    make_builtin_fn!(group, named "groupCollapsed", of console);
    make_builtin_fn!(group_end , named "groupEnd", of console);
    make_builtin_fn!(time, named "time", of console);
    make_builtin_fn!(time_log, named "timeLog", of console);
    make_builtin_fn!(time_end, named "timeEnd", of console);
    make_builtin_fn!(dir, named "dir", of console);
    make_builtin_fn!(dir, named "dirxml", of console);

    console.set_internal_state(ConsoleState::default());

    console
}

/// Initialise the `console` object on the global object.
#[inline]
pub fn init(global: &Value) {
    global.set_field_slice("console", create(global));
}
