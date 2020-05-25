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
        function::make_builtin_fn,
        object::InternalState,
        value::{display_obj, ResultValue, Value},
    },
    exec::Interpreter,
    BoaProfiler,
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
fn get_arg_at_index<'a, T>(args: &'a [Value], index: usize) -> Option<T>
where
    T: From<&'a Value> + Default,
{
    args.get(index).map(|s| T::from(s))
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
pub fn formatter(data: &[Value], ctx: &mut Interpreter) -> Result<String, Value> {
    let target = ctx.to_string(&data.get(0).cloned().unwrap_or_default())?;
    match data.len() {
        0 => Ok(String::new()),
        1 => Ok(target),
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
                                ctx.to_string(&data.get(arg_index).cloned().unwrap_or_default())?;
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

            Ok(formatted)
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
pub fn assert(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    let assertion = get_arg_at_index::<bool>(args, 0).unwrap_or_default();

    if !assertion {
        let mut args: Vec<Value> = args.iter().skip(1).cloned().collect();
        let message = "Assertion failed".to_string();
        if args.is_empty() {
            args.push(Value::from(message));
        } else if !args[0].is_string() {
            args.insert(0, Value::from(message));
        } else {
            let concat = format!("{}: {}", message, args[0]);
            args[0] = Value::from(concat);
        }

        this.with_internal_state_ref::<_, Result<(), Value>, _>(|state| {
            logger(LogMessage::Error(formatter(&args, ctx)?), state);
            Ok(())
        })?;
    }

    Ok(Value::undefined())
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

    Ok(Value::undefined())
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
pub fn debug(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref::<_, Result<(), Value>, _>(|state| {
        logger(LogMessage::Log(formatter(args, ctx)?), state);
        Ok(())
    })?;
    Ok(Value::undefined())
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
pub fn error(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref::<_, Result<(), Value>, _>(|state| {
        logger(LogMessage::Error(formatter(args, ctx)?), state);
        Ok(())
    })?;
    Ok(Value::undefined())
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
pub fn info(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref::<_, Result<(), Value>, _>(|state| {
        logger(LogMessage::Info(formatter(args, ctx)?), state);
        Ok(())
    })?;
    Ok(Value::undefined())
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
pub fn log(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref::<_, Result<(), Value>, _>(|state| {
        logger(LogMessage::Log(formatter(args, ctx)?), state);
        Ok(())
    })?;
    Ok(Value::undefined())
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
pub fn trace(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    if !args.is_empty() {
        this.with_internal_state_ref::<_, Result<(), Value>, _>(|state| {
            logger(LogMessage::Log(formatter(args, ctx)?), state);
            Ok(())
        })?;

        /* TODO: get and print stack trace */
        this.with_internal_state_ref(|state| {
            logger(
                LogMessage::Log("Not implemented: <stack trace>".to_string()),
                state,
            )
        });
    }

    Ok(Value::undefined())
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
pub fn warn(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    this.with_internal_state_ref::<_, Result<(), Value>, _>(|state| {
        logger(LogMessage::Warn(formatter(args, ctx)?), state);
        Ok(())
    })?;
    Ok(Value::undefined())
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
pub fn count(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    let label = match args.get(0) {
        Some(value) => ctx.to_string(value)?,
        None => "default".to_owned(),
    };

    this.with_internal_state_mut(|state: &mut ConsoleState| {
        let msg = format!("count {}:", &label);
        let c = state.count_map.entry(label).or_insert(0);
        *c += 1;

        logger(LogMessage::Info(format!("{} {}", msg, c)), state);
    });

    Ok(Value::undefined())
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
pub fn count_reset(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    let label = match args.get(0) {
        Some(value) => ctx.to_string(value)?,
        None => "default".to_owned(),
    };

    this.with_internal_state_mut(|state: &mut ConsoleState| {
        state.count_map.remove(&label);

        logger(LogMessage::Warn(format!("countReset {}", label)), state);
    });

    Ok(Value::undefined())
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
pub fn time(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    let label = match args.get(0) {
        Some(value) => ctx.to_string(value)?,
        None => "default".to_owned(),
    };

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

    Ok(Value::undefined())
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
pub fn time_log(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    let label = match args.get(0) {
        Some(value) => ctx.to_string(value)?,
        None => "default".to_owned(),
    };

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

    Ok(Value::undefined())
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
pub fn time_end(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    let label = match args.get(0) {
        Some(value) => ctx.to_string(value)?,
        None => "default".to_owned(),
    };

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

    Ok(Value::undefined())
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
pub fn group(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    let group_label = formatter(args, ctx)?;

    this.with_internal_state_mut(|state: &mut ConsoleState| {
        logger(LogMessage::Info(format!("group: {}", &group_label)), state);
        state.groups.push(group_label);
    });

    Ok(Value::undefined())
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

    Ok(Value::undefined())
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
        let undefined = Value::undefined();
        logger(
            LogMessage::Info(display_obj(args.get(0).unwrap_or(&undefined), true)),
            state,
        );
    });

    Ok(Value::undefined())
}

/// Create a new `console` object
pub fn create(global: &Value) -> Value {
    let console = Value::new_object(Some(global));

    make_builtin_fn(assert, "assert", &console, 0);
    make_builtin_fn(clear, "clear", &console, 0);
    make_builtin_fn(debug, "debug", &console, 0);
    make_builtin_fn(error, "error", &console, 0);
    make_builtin_fn(info, "info", &console, 0);
    make_builtin_fn(log, "log", &console, 0);
    make_builtin_fn(trace, "trace", &console, 0);
    make_builtin_fn(warn, "warn", &console, 0);
    make_builtin_fn(error, "exception", &console, 0);
    make_builtin_fn(count, "count", &console, 0);
    make_builtin_fn(count_reset, "countReset", &console, 0);
    make_builtin_fn(group, "group", &console, 0);
    make_builtin_fn(group, "groupCollapsed", &console, 0);
    make_builtin_fn(group_end, "groupEnd", &console, 0);
    make_builtin_fn(time, "time", &console, 0);
    make_builtin_fn(time_log, "timeLog", &console, 0);
    make_builtin_fn(time_end, "timeEnd", &console, 0);
    make_builtin_fn(dir, "dir", &console, 0);
    make_builtin_fn(dir, "dirxml", &console, 0);

    console.set_internal_state(ConsoleState::default());

    console
}

/// Initialise the `console` object on the global object.
#[inline]
pub fn init(global: &Value) {
    let _timer = BoaProfiler::global().start_event("console", "init");

    let console = create(global);
    global
        .as_object_mut()
        .unwrap()
        .insert_field("console", console);
}
