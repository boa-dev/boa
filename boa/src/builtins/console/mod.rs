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
    builtins::BuiltIn,
    object::ObjectInitializer,
    property::Attribute,
    value::{display_obj, RcString, Value},
    BoaProfiler, Context, Result,
};
use rustc_hash::FxHashMap;
use std::time::SystemTime;

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
pub(crate) fn logger(msg: LogMessage, console_state: &Console) {
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
pub fn formatter(data: &[Value], ctx: &mut Context) -> Result<String> {
    let target = data.get(0).cloned().unwrap_or_default().to_string(ctx)?;

    match data.len() {
        0 => Ok(String::new()),
        1 => Ok(target.to_string()),
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
                            let arg = data
                                .get(arg_index)
                                .cloned()
                                .unwrap_or_default()
                                .to_integer(ctx)?;
                            formatted.push_str(&format!("{}", arg));
                            arg_index += 1;
                        }
                        /* float */
                        'f' => {
                            let arg = data
                                .get(arg_index)
                                .cloned()
                                .unwrap_or_default()
                                .to_number(ctx)?;
                            formatted.push_str(&format!("{number:.prec$}", number = arg, prec = 6));
                            arg_index += 1
                        }
                        /* object, FIXME: how to render this properly? */
                        'o' | 'O' => {
                            let arg = data.get(arg_index).cloned().unwrap_or_default();
                            formatted.push_str(&format!("{}", arg.display()));
                            arg_index += 1
                        }
                        /* string */
                        's' => {
                            let arg = data
                                .get(arg_index)
                                .cloned()
                                .unwrap_or_default()
                                .to_string(ctx)?;
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
                formatted.push_str(&format!(" {}", rest.to_string(ctx)?))
            }

            Ok(formatted)
        }
    }
}

/// This is the internal console object state.
#[derive(Debug, Default)]
pub(crate) struct Console {
    count_map: FxHashMap<RcString, u32>,
    timer_map: FxHashMap<RcString, u128>,
    groups: Vec<String>,
}

impl BuiltIn for Console {
    const NAME: &'static str = "console";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, Value, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");
        let console = ObjectInitializer::new(context)
            .function(Self::assert, "assert", 0)
            .function(Self::clear, "clear", 0)
            .function(Self::debug, "debug", 0)
            .function(Self::error, "error", 0)
            .function(Self::info, "info", 0)
            .function(Self::log, "log", 0)
            .function(Self::trace, "trace", 0)
            .function(Self::warn, "warn", 0)
            .function(Self::error, "exception", 0)
            .function(Self::count, "count", 0)
            .function(Self::count_reset, "countReset", 0)
            .function(Self::group, "group", 0)
            .function(Self::group, "groupCollapsed", 0)
            .function(Self::group_end, "groupEnd", 0)
            .function(Self::time, "time", 0)
            .function(Self::time_log, "timeLog", 0)
            .function(Self::time_end, "timeEnd", 0)
            .function(Self::dir, "dir", 0)
            .function(Self::dir, "dirxml", 0)
            .build();

        (Self::NAME, console.into(), Self::attribute())
    }
}

impl Console {
    /// The name of the object.
    pub(crate) const NAME: &'static str = "console";

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
    pub(crate) fn assert(_: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        let assertion = get_arg_at_index::<bool>(args, 0).unwrap_or_default();

        if !assertion {
            let mut args: Vec<Value> = args.iter().skip(1).cloned().collect();
            let message = "Assertion failed".to_string();
            if args.is_empty() {
                args.push(Value::from(message));
            } else if !args[0].is_string() {
                args.insert(0, Value::from(message));
            } else {
                let concat = format!("{}: {}", message, args[0].display());
                args[0] = Value::from(concat);
            }

            logger(LogMessage::Error(formatter(&args, ctx)?), ctx.console());
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
    pub(crate) fn clear(_: &Value, _: &[Value], ctx: &mut Context) -> Result<Value> {
        ctx.console_mut().groups.clear();
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
    pub(crate) fn debug(_: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        logger(LogMessage::Log(formatter(args, ctx)?), ctx.console());
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
    pub(crate) fn error(_: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        logger(LogMessage::Error(formatter(args, ctx)?), ctx.console());
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
    pub(crate) fn info(_: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        logger(LogMessage::Info(formatter(args, ctx)?), ctx.console());
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
    pub(crate) fn log(_: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        logger(LogMessage::Log(formatter(args, ctx)?), ctx.console());
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
    pub(crate) fn trace(_: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        if !args.is_empty() {
            logger(LogMessage::Log(formatter(args, ctx)?), ctx.console());

            /* TODO: get and print stack trace */
            logger(
                LogMessage::Log("Not implemented: <stack trace>".to_string()),
                ctx.console(),
            )
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
    pub(crate) fn warn(_: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        logger(LogMessage::Warn(formatter(args, ctx)?), ctx.console());
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
    pub(crate) fn count(_: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        let label = match args.get(0) {
            Some(value) => value.to_string(ctx)?,
            None => "default".into(),
        };

        let msg = format!("count {}:", &label);
        let c = ctx.console_mut().count_map.entry(label).or_insert(0);
        *c += 1;

        logger(LogMessage::Info(format!("{} {}", msg, c)), ctx.console());
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
    pub(crate) fn count_reset(_: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        let label = match args.get(0) {
            Some(value) => value.to_string(ctx)?,
            None => "default".into(),
        };

        ctx.console_mut().count_map.remove(&label);

        logger(
            LogMessage::Warn(format!("countReset {}", label)),
            ctx.console(),
        );

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
    pub(crate) fn time(_: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        let label = match args.get(0) {
            Some(value) => value.to_string(ctx)?,
            None => "default".into(),
        };

        if ctx.console().timer_map.get(&label).is_some() {
            logger(
                LogMessage::Warn(format!("Timer '{}' already exist", label)),
                ctx.console(),
            );
        } else {
            let time = Self::system_time_in_ms();
            ctx.console_mut().timer_map.insert(label, time);
        }

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
    pub(crate) fn time_log(_: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        let label = match args.get(0) {
            Some(value) => value.to_string(ctx)?,
            None => "default".into(),
        };

        if let Some(t) = ctx.console().timer_map.get(&label) {
            let time = Self::system_time_in_ms();
            let mut concat = format!("{}: {} ms", label, time - t);
            for msg in args.iter().skip(1) {
                concat = concat + " " + &msg.display().to_string();
            }
            logger(LogMessage::Log(concat), ctx.console());
        } else {
            logger(
                LogMessage::Warn(format!("Timer '{}' doesn't exist", label)),
                ctx.console(),
            );
        }

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
    pub(crate) fn time_end(_: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        let label = match args.get(0) {
            Some(value) => value.to_string(ctx)?,
            None => "default".into(),
        };

        if let Some(t) = ctx.console_mut().timer_map.remove(label.as_str()) {
            let time = Self::system_time_in_ms();
            logger(
                LogMessage::Info(format!("{}: {} ms - timer removed", label, time - t)),
                ctx.console(),
            );
        } else {
            logger(
                LogMessage::Warn(format!("Timer '{}' doesn't exist", label)),
                ctx.console(),
            );
        }

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
    pub(crate) fn group(_: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        let group_label = formatter(args, ctx)?;

        logger(
            LogMessage::Info(format!("group: {}", &group_label)),
            ctx.console(),
        );
        ctx.console_mut().groups.push(group_label);

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
    pub(crate) fn group_end(_: &Value, _: &[Value], ctx: &mut Context) -> Result<Value> {
        ctx.console_mut().groups.pop();

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
    pub(crate) fn dir(_: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        let undefined = Value::undefined();
        logger(
            LogMessage::Info(display_obj(args.get(0).unwrap_or(&undefined), true)),
            ctx.console(),
        );

        Ok(Value::undefined())
    }
}
