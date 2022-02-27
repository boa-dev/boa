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
    builtins::{BuiltIn, JsArgs},
    object::ObjectInitializer,
    value::{display::display_obj, JsValue, Numeric},
    Context, JsResult, JsString,
};
use boa_profiler::Profiler;
use rustc_hash::FxHashMap;
use std::time::SystemTime;
use tap::{Conv, Pipe};

/// This represents the different types of log messages.
#[derive(Debug)]
pub enum LogMessage {
    Log(String),
    Info(String),
    Warn(String),
    Error(String),
}

/// Helper function for logging messages.
pub(crate) fn logger(msg: LogMessage, console_state: &Console) {
    let indent = 2 * console_state.groups.len();

    match msg {
        LogMessage::Error(msg) => {
            eprintln!("{msg:>indent$}");
        }
        LogMessage::Log(msg) | LogMessage::Info(msg) | LogMessage::Warn(msg) => {
            println!("{msg:>indent$}");
        }
    }
}

/// This represents the `console` formatter.
pub fn formatter(data: &[JsValue], context: &mut Context) -> JsResult<String> {
    let target = data
        .get(0)
        .cloned()
        .unwrap_or_default()
        .to_string(context)?;

    match data.len() {
        0 => Ok(String::new()),
        1 => Ok(target.to_std_string_escaped()),
        _ => {
            let mut formatted = String::new();
            let mut arg_index = 1;
            let target = target.to_std_string_escaped();
            let mut chars = target.chars();
            while let Some(c) = chars.next() {
                if c == '%' {
                    let fmt = chars.next().unwrap_or('%');
                    match fmt {
                        /* integer */
                        'd' | 'i' => {
                            let arg = match data.get_or_undefined(arg_index).to_numeric(context)? {
                                Numeric::Number(r) => (r.floor() + 0.0).to_string(),
                                Numeric::BigInt(int) => int.to_string(),
                            };
                            formatted.push_str(&arg);
                            arg_index += 1;
                        }
                        /* float */
                        'f' => {
                            let arg = data.get_or_undefined(arg_index).to_number(context)?;
                            formatted.push_str(&format!("{arg:.6}"));
                            arg_index += 1;
                        }
                        /* object, FIXME: how to render this properly? */
                        'o' | 'O' => {
                            let arg = data.get_or_undefined(arg_index);
                            formatted.push_str(&arg.display().to_string());
                            arg_index += 1;
                        }
                        /* string */
                        's' => {
                            let arg = data
                                .get(arg_index)
                                .cloned()
                                .unwrap_or_default()
                                .to_string(context)?
                                .to_std_string_escaped();
                            formatted.push_str(&arg);
                            arg_index += 1;
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
                formatted.push_str(&format!(
                    " {}",
                    rest.to_string(context)?.to_std_string_escaped()
                ));
            }

            Ok(formatted)
        }
    }
}

/// This is the internal console object state.
#[derive(Debug, Default)]
pub(crate) struct Console {
    count_map: FxHashMap<JsString, u32>,
    timer_map: FxHashMap<JsString, u128>,
    groups: Vec<String>,
}

impl BuiltIn for Console {
    const NAME: &'static str = "console";

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");
        ObjectInitializer::new(context)
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
            .build()
            .conv::<JsValue>()
            .pipe(Some)
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
    pub(crate) fn assert(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let assertion = args.get(0).map_or(false, JsValue::to_boolean);

        if !assertion {
            let mut args: Vec<JsValue> = args.iter().skip(1).cloned().collect();
            let message = "Assertion failed".to_string();
            if args.is_empty() {
                args.push(JsValue::new(message));
            } else if !args[0].is_string() {
                args.insert(0, JsValue::new(message));
            } else {
                let concat = format!("{message}: {}", args[0].display());
                args[0] = JsValue::new(concat);
            }

            logger(
                LogMessage::Error(formatter(&args, context)?),
                context.console(),
            );
        }

        Ok(JsValue::undefined())
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
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn clear(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        context.console_mut().groups.clear();
        Ok(JsValue::undefined())
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
    pub(crate) fn debug(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        logger(
            LogMessage::Log(formatter(args, context)?),
            context.console(),
        );
        Ok(JsValue::undefined())
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
    pub(crate) fn error(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        logger(
            LogMessage::Error(formatter(args, context)?),
            context.console(),
        );
        Ok(JsValue::undefined())
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
    pub(crate) fn info(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        logger(
            LogMessage::Info(formatter(args, context)?),
            context.console(),
        );
        Ok(JsValue::undefined())
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
    pub(crate) fn log(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        logger(
            LogMessage::Log(formatter(args, context)?),
            context.console(),
        );
        Ok(JsValue::undefined())
    }

    fn get_stack_trace(context: &mut Context) -> Vec<String> {
        let mut stack_trace: Vec<String> = vec![];

        for frame in context.vm.frames.iter().rev() {
            stack_trace.push(
                context
                    .interner()
                    .resolve_expect(frame.code.name)
                    .to_string(),
            );
        }

        stack_trace
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
    pub(crate) fn trace(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if !args.is_empty() {
            logger(
                LogMessage::Log(formatter(args, context)?),
                context.console(),
            );

            let stack_trace_dump = Self::get_stack_trace(context).join("\n");
            logger(LogMessage::Log(stack_trace_dump), context.console());
        }

        Ok(JsValue::undefined())
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
    pub(crate) fn warn(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        logger(
            LogMessage::Warn(formatter(args, context)?),
            context.console(),
        );
        Ok(JsValue::undefined())
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
    pub(crate) fn count(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let label = match args.get(0) {
            Some(value) => value.to_string(context)?,
            None => "default".into(),
        };

        let msg = format!("count {}:", label.to_std_string_escaped());
        let c = context.console_mut().count_map.entry(label).or_insert(0);
        *c += 1;

        logger(LogMessage::Info(format!("{msg} {c}")), context.console());
        Ok(JsValue::undefined())
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
    pub(crate) fn count_reset(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let label = match args.get(0) {
            Some(value) => value.to_string(context)?,
            None => "default".into(),
        };

        context.console_mut().count_map.remove(&label);

        logger(
            LogMessage::Warn(format!("countReset {}", label.to_std_string_escaped())),
            context.console(),
        );

        Ok(JsValue::undefined())
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
    pub(crate) fn time(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let label = match args.get(0) {
            Some(value) => value.to_string(context)?,
            None => "default".into(),
        };

        if context.console().timer_map.get(&label).is_some() {
            logger(
                LogMessage::Warn(format!(
                    "Timer '{}' already exist",
                    label.to_std_string_escaped()
                )),
                context.console(),
            );
        } else {
            let time = Self::system_time_in_ms();
            context.console_mut().timer_map.insert(label, time);
        }

        Ok(JsValue::undefined())
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
    pub(crate) fn time_log(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let label = match args.get(0) {
            Some(value) => value.to_string(context)?,
            None => "default".into(),
        };

        if let Some(t) = context.console().timer_map.get(&label) {
            let time = Self::system_time_in_ms();
            let mut concat = format!("{}: {} ms", label.to_std_string_escaped(), time - t);
            for msg in args.iter().skip(1) {
                concat = concat + " " + &msg.display().to_string();
            }
            logger(LogMessage::Log(concat), context.console());
        } else {
            logger(
                LogMessage::Warn(format!(
                    "Timer '{}' doesn't exist",
                    label.to_std_string_escaped()
                )),
                context.console(),
            );
        }

        Ok(JsValue::undefined())
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
    pub(crate) fn time_end(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let label = match args.get(0) {
            Some(value) => value.to_string(context)?,
            None => "default".into(),
        };

        if let Some(t) = context.console_mut().timer_map.remove(&label) {
            let time = Self::system_time_in_ms();
            logger(
                LogMessage::Info(format!(
                    "{}: {} ms - timer removed",
                    label.to_std_string_escaped(),
                    time - t
                )),
                context.console(),
            );
        } else {
            logger(
                LogMessage::Warn(format!(
                    "Timer '{}' doesn't exist",
                    label.to_std_string_escaped()
                )),
                context.console(),
            );
        }

        Ok(JsValue::undefined())
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
    pub(crate) fn group(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let group_label = formatter(args, context)?;

        logger(
            LogMessage::Info(format!("group: {group_label}")),
            context.console(),
        );
        context.console_mut().groups.push(group_label);

        Ok(JsValue::undefined())
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
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn group_end(
        _: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        context.console_mut().groups.pop();

        Ok(JsValue::undefined())
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
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn dir(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        logger(
            LogMessage::Info(display_obj(args.get_or_undefined(0), true)),
            context.console(),
        );
        Ok(JsValue::undefined())
    }
}
