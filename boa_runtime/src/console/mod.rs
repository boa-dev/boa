//! Boa's implementation of JavaScript's `console` Web API object.
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

#[cfg(test)]
mod tests;

use boa_engine::{
    native_function::NativeFunction,
    object::{JsObject, ObjectInitializer},
    value::{JsValue, Numeric},
    Context, JsArgs, JsResult, JsString,
};
use boa_gc::{Finalize, Trace};
// use boa_profiler::Profiler;
use rustc_hash::FxHashMap;
use std::{cell::RefCell, rc::Rc, time::SystemTime};

/// This represents the different types of log messages.
#[derive(Debug)]
enum LogMessage {
    Log(String),
    Info(String),
    Warn(String),
    Error(String),
}

/// Helper function for logging messages.
fn logger(msg: LogMessage, console_state: &Console) {
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
fn formatter(data: &[JsValue], context: &mut Context<'_>) -> JsResult<String> {
    match data {
        [] => Ok(String::new()),
        [val] => Ok(val.to_string(context)?.to_std_string_escaped()),
        data => {
            let mut formatted = String::new();
            let mut arg_index = 1;
            let target = data
                .get_or_undefined(0)
                .to_string(context)?
                .to_std_string_escaped();
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
                                .get_or_undefined(arg_index)
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
#[derive(Debug, Default, Trace, Finalize)]
pub struct Console {
    count_map: FxHashMap<JsString, u32>,
    timer_map: FxHashMap<JsString, u128>,
    groups: Vec<String>,
}

impl Console {
    /// Name of the built-in `console` property.
    pub const NAME: &'static str = "console";

    /// Initializes the `console` built-in object.
    pub fn init(context: &mut Context<'_>) -> JsObject {
        fn console_method(
            f: fn(&JsValue, &[JsValue], &Console, &mut Context<'_>) -> JsResult<JsValue>,
            state: Rc<RefCell<Console>>,
        ) -> NativeFunction {
            // SAFETY: `Console` doesn't contain types that need tracing.
            unsafe {
                NativeFunction::from_closure(move |this, args, context| {
                    f(this, args, &state.borrow(), context)
                })
            }
        }
        fn console_method_mut(
            f: fn(&JsValue, &[JsValue], &mut Console, &mut Context<'_>) -> JsResult<JsValue>,
            state: Rc<RefCell<Console>>,
        ) -> NativeFunction {
            // SAFETY: `Console` doesn't contain types that need tracing.
            unsafe {
                NativeFunction::from_closure(move |this, args, context| {
                    f(this, args, &mut state.borrow_mut(), context)
                })
            }
        }
        // let _timer = Profiler::global().start_event(Self::NAME, "init");

        let state = Rc::new(RefCell::new(Self::default()));

        ObjectInitializer::with_native(Self::default(), context)
            .function(console_method(Self::assert, state.clone()), "assert", 0)
            .function(console_method_mut(Self::clear, state.clone()), "clear", 0)
            .function(console_method(Self::debug, state.clone()), "debug", 0)
            .function(console_method(Self::error, state.clone()), "error", 0)
            .function(console_method(Self::info, state.clone()), "info", 0)
            .function(console_method(Self::log, state.clone()), "log", 0)
            .function(console_method(Self::trace, state.clone()), "trace", 0)
            .function(console_method(Self::warn, state.clone()), "warn", 0)
            .function(console_method_mut(Self::count, state.clone()), "count", 0)
            .function(
                console_method_mut(Self::count_reset, state.clone()),
                "countReset",
                0,
            )
            .function(console_method_mut(Self::group, state.clone()), "group", 0)
            .function(
                console_method_mut(Self::group, state.clone()),
                "groupCollapsed",
                0,
            )
            .function(
                console_method_mut(Self::group_end, state.clone()),
                "groupEnd",
                0,
            )
            .function(console_method_mut(Self::time, state.clone()), "time", 0)
            .function(console_method(Self::time_log, state.clone()), "timeLog", 0)
            .function(
                console_method_mut(Self::time_end, state.clone()),
                "timeEnd",
                0,
            )
            .function(console_method(Self::dir, state.clone()), "dir", 0)
            .function(console_method(Self::dir, state), "dirxml", 0)
            .build()
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
    fn assert(
        _: &JsValue,
        args: &[JsValue],
        console: &Self,
        context: &mut Context<'_>,
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

            logger(LogMessage::Error(formatter(&args, context)?), console);
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
    fn clear(
        _: &JsValue,
        _: &[JsValue],
        console: &mut Self,
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        console.groups.clear();
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
    fn debug(
        _: &JsValue,
        args: &[JsValue],
        console: &Self,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        logger(LogMessage::Log(formatter(args, context)?), console);
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
    fn error(
        _: &JsValue,
        args: &[JsValue],
        console: &Self,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        logger(LogMessage::Error(formatter(args, context)?), console);
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
    fn info(
        _: &JsValue,
        args: &[JsValue],
        console: &Self,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        logger(LogMessage::Info(formatter(args, context)?), console);
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
    fn log(
        _: &JsValue,
        args: &[JsValue],
        console: &Self,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        logger(LogMessage::Log(formatter(args, context)?), console);
        Ok(JsValue::undefined())
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
    fn trace(
        _: &JsValue,
        args: &[JsValue],
        console: &Self,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        if !args.is_empty() {
            logger(LogMessage::Log(formatter(args, context)?), console);

            let stack_trace_dump = context
                .stack_trace()
                .map(|frame| frame.code_block().name())
                .collect::<Vec<_>>()
                .into_iter()
                .map(|s| context.interner().resolve_expect(s).to_string())
                .collect::<Vec<_>>()
                .join("\n");
            logger(LogMessage::Log(stack_trace_dump), console);
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
    fn warn(
        _: &JsValue,
        args: &[JsValue],
        console: &Self,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        logger(LogMessage::Warn(formatter(args, context)?), console);
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
    fn count(
        _: &JsValue,
        args: &[JsValue],
        console: &mut Self,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let label = match args.get(0) {
            Some(value) => value.to_string(context)?,
            None => "default".into(),
        };

        let msg = format!("count {}:", label.to_std_string_escaped());
        let c = console.count_map.entry(label).or_insert(0);
        *c += 1;

        logger(LogMessage::Info(format!("{msg} {c}")), console);
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
    fn count_reset(
        _: &JsValue,
        args: &[JsValue],
        console: &mut Self,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let label = match args.get(0) {
            Some(value) => value.to_string(context)?,
            None => "default".into(),
        };

        console.count_map.remove(&label);

        logger(
            LogMessage::Warn(format!("countReset {}", label.to_std_string_escaped())),
            console,
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
    fn time(
        _: &JsValue,
        args: &[JsValue],
        console: &mut Self,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let label = match args.get(0) {
            Some(value) => value.to_string(context)?,
            None => "default".into(),
        };

        if console.timer_map.get(&label).is_some() {
            logger(
                LogMessage::Warn(format!(
                    "Timer '{}' already exist",
                    label.to_std_string_escaped()
                )),
                console,
            );
        } else {
            let time = Self::system_time_in_ms();
            console.timer_map.insert(label, time);
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
    fn time_log(
        _: &JsValue,
        args: &[JsValue],
        console: &Self,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let label = match args.get(0) {
            Some(value) => value.to_string(context)?,
            None => "default".into(),
        };

        console.timer_map.get(&label).map_or_else(
            || {
                logger(
                    LogMessage::Warn(format!(
                        "Timer '{}' doesn't exist",
                        label.to_std_string_escaped()
                    )),
                    console,
                );
            },
            |t| {
                let time = Self::system_time_in_ms();
                let mut concat = format!("{}: {} ms", label.to_std_string_escaped(), time - t);
                for msg in args.iter().skip(1) {
                    concat = concat + " " + &msg.display().to_string();
                }
                logger(LogMessage::Log(concat), console);
            },
        );

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
    fn time_end(
        _: &JsValue,
        args: &[JsValue],
        console: &mut Self,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let label = match args.get(0) {
            Some(value) => value.to_string(context)?,
            None => "default".into(),
        };

        console.timer_map.remove(&label).map_or_else(
            || {
                logger(
                    LogMessage::Warn(format!(
                        "Timer '{}' doesn't exist",
                        label.to_std_string_escaped()
                    )),
                    console,
                );
            },
            |t| {
                let time = Self::system_time_in_ms();
                logger(
                    LogMessage::Info(format!(
                        "{}: {} ms - timer removed",
                        label.to_std_string_escaped(),
                        time - t
                    )),
                    console,
                );
            },
        );

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
    fn group(
        _: &JsValue,
        args: &[JsValue],
        console: &mut Self,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let group_label = formatter(args, context)?;

        logger(LogMessage::Info(format!("group: {group_label}")), console);
        console.groups.push(group_label);

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
    fn group_end(
        _: &JsValue,
        _: &[JsValue],
        console: &mut Self,
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        console.groups.pop();

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
    fn dir(
        _: &JsValue,
        args: &[JsValue],
        console: &Self,
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        logger(
            LogMessage::Info(args.get_or_undefined(0).display_obj(true)),
            console,
        );
        Ok(JsValue::undefined())
    }
}
