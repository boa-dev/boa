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

use boa_engine::property::Attribute;
use boa_engine::{
    js_str, js_string,
    native_function::NativeFunction,
    object::{JsObject, ObjectInitializer},
    value::{JsValue, Numeric},
    Context, JsArgs, JsData, JsError, JsResult, JsStr, JsString, JsSymbol,
};
use boa_gc::{Finalize, Trace};
use rustc_hash::FxHashMap;
use std::{cell::RefCell, collections::hash_map::Entry, io::Write, rc::Rc, time::SystemTime};

/// A trait that can be used to forward console logs to an implementation.
pub trait Logger: Trace + Sized {
    /// Log a log message (`console.log`).
    ///
    /// # Errors
    /// Returning an error will throw an exception in JavaScript.
    fn log(&self, msg: String, state: &ConsoleState, context: &mut Context) -> JsResult<()>;

    /// Log an info message (`console.info`).
    ///
    /// # Errors
    /// Returning an error will throw an exception in JavaScript.
    fn info(&self, msg: String, state: &ConsoleState, context: &mut Context) -> JsResult<()>;

    /// Log a warning message (`console.warn`).
    ///
    /// # Errors
    /// Returning an error will throw an exception in JavaScript.
    fn warn(&self, msg: String, state: &ConsoleState, context: &mut Context) -> JsResult<()>;

    /// Log an error message (`console.error`).
    ///
    /// # Errors
    /// Returning an error will throw an exception in JavaScript.
    fn error(&self, msg: String, state: &ConsoleState, context: &mut Context) -> JsResult<()>;
}

/// The default implementation for logging from the console.
///
/// Implements the [`Logger`] trait and output errors to stderr and all
/// the others to stdout. Will add indentation based on the number of
/// groups.
#[derive(Debug, Trace, Finalize)]
pub struct DefaultLogger;

impl Logger for DefaultLogger {
    #[inline]
    fn log(&self, msg: String, state: &ConsoleState, _context: &mut Context) -> JsResult<()> {
        let indent = state.indent();
        writeln!(std::io::stdout(), "{msg:>indent$}").map_err(JsError::from_rust)
    }

    #[inline]
    fn info(&self, msg: String, state: &ConsoleState, context: &mut Context) -> JsResult<()> {
        self.log(msg, state, context)
    }

    #[inline]
    fn warn(&self, msg: String, state: &ConsoleState, context: &mut Context) -> JsResult<()> {
        self.log(msg, state, context)
    }

    #[inline]
    fn error(&self, msg: String, state: &ConsoleState, _context: &mut Context) -> JsResult<()> {
        let indent = state.indent();
        writeln!(std::io::stderr(), "{msg:>indent$}").map_err(JsError::from_rust)
    }
}

/// This represents the `console` formatter.
fn formatter(data: &[JsValue], context: &mut Context) -> JsResult<String> {
    fn to_string(value: &JsValue, context: &mut Context) -> JsResult<String> {
        // There is a slight difference between the standard [`JsValue::to_string`] and
        // the way Console actually logs, w.r.t Symbols.
        if let Some(s) = value.as_symbol() {
            Ok(s.to_string())
        } else {
            Ok(value.to_string(context)?.to_std_string_escaped())
        }
    }

    match data {
        [] => Ok(String::new()),
        [val] => to_string(val, context),
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
                            let arg = data.get_or_undefined(arg_index);

                            // If a JS value implements `toString()`, call it.
                            let mut written = false;
                            if let Some(obj) = arg.as_object() {
                                if let Ok(to_string) = obj.get(js_str!("toString"), context) {
                                    if let Some(to_string_fn) = to_string.as_function() {
                                        let arg = to_string_fn
                                            .call(arg, &[], context)?
                                            .to_string(context)?;
                                        formatted.push_str(&arg.to_std_string_escaped());
                                        written = true;
                                    }
                                }
                            }

                            if !written {
                                let arg = arg.to_string(context)?.to_std_string_escaped();
                                formatted.push_str(&arg);
                            }

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
                formatted.push(' ');
                formatted.push_str(&to_string(rest, context)?);
            }

            Ok(formatted)
        }
    }
}

/// The current state of the console, passed to the logger backend.
/// This should not be copied or cloned. References are only valid
/// for the current logging call.
#[derive(Debug, Default, Trace, Finalize)]
pub struct ConsoleState {
    /// The map of console counters, used in `console.count()`.
    count_map: FxHashMap<JsString, u32>,

    /// The map of console timers, used in `console.time`, `console.timeLog`
    /// and `console.timeEnd`.
    timer_map: FxHashMap<JsString, u128>,

    /// The current list of groups. Groups should be indented, but some logging
    /// libraries may want to use them in a different way.
    groups: Vec<String>,
}

impl ConsoleState {
    /// Returns the indentation level that should be applied to logging.
    #[must_use]
    pub fn indent(&self) -> usize {
        2 * self.groups.len()
    }

    /// Returns the current list of groups.
    #[must_use]
    pub fn groups(&self) -> &Vec<String> {
        &self.groups
    }

    /// Returns the count map.
    #[must_use]
    pub fn count_map(&self) -> &FxHashMap<JsString, u32> {
        &self.count_map
    }

    /// Returns the timer map.
    #[must_use]
    pub fn timer_map(&self) -> &FxHashMap<JsString, u128> {
        &self.timer_map
    }
}

/// This is the internal console object state.
#[derive(Debug, Default, Trace, Finalize, JsData)]
pub struct Console {
    state: ConsoleState,
}

impl Console {
    /// Name of the built-in `console` property.
    pub const NAME: JsStr<'static> = js_str!("console");

    /// Modify the context to include the `console` object.
    ///
    /// # Errors
    /// This function will return an error if the property cannot be defined on the global object.
    pub fn register_with_logger<L>(context: &mut Context, logger: L) -> JsResult<()>
    where
        L: Logger + 'static,
    {
        let console = Self::init_with_logger(context, logger);
        context.register_global_property(
            Self::NAME,
            console,
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )?;

        Ok(())
    }

    /// Initializes the `console` with a special logger.
    #[allow(clippy::too_many_lines)]
    pub fn init_with_logger<L>(context: &mut Context, logger: L) -> JsObject
    where
        L: Logger + 'static,
    {
        fn console_method<L: Logger + 'static>(
            f: fn(&JsValue, &[JsValue], &Console, &L, &mut Context) -> JsResult<JsValue>,
            state: Rc<RefCell<Console>>,
            logger: Rc<L>,
        ) -> NativeFunction {
            // SAFETY: `Console` doesn't contain types that need tracing.
            unsafe {
                NativeFunction::from_closure(move |this, args, context| {
                    f(this, args, &state.borrow(), &logger, context)
                })
            }
        }
        fn console_method_mut<L: Logger + 'static>(
            f: fn(&JsValue, &[JsValue], &mut Console, &L, &mut Context) -> JsResult<JsValue>,
            state: Rc<RefCell<Console>>,
            logger: Rc<L>,
        ) -> NativeFunction {
            // SAFETY: `Console` doesn't contain types that need tracing.
            unsafe {
                NativeFunction::from_closure(move |this, args, context| {
                    f(this, args, &mut state.borrow_mut(), &logger, context)
                })
            }
        }
        // let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        let state = Rc::new(RefCell::new(Self::default()));
        let logger = Rc::new(logger);

        ObjectInitializer::with_native_data_and_proto(
            Self::default(),
            JsObject::with_object_proto(context.realm().intrinsics()),
            context,
        )
        .property(
            JsSymbol::to_string_tag(),
            Self::NAME,
            Attribute::CONFIGURABLE,
        )
        .function(
            console_method(Self::assert, state.clone(), logger.clone()),
            js_string!("assert"),
            0,
        )
        .function(
            console_method_mut(Self::clear, state.clone(), logger.clone()),
            js_string!("clear"),
            0,
        )
        .function(
            console_method(Self::debug, state.clone(), logger.clone()),
            js_string!("debug"),
            0,
        )
        .function(
            console_method(Self::error, state.clone(), logger.clone()),
            js_string!("error"),
            0,
        )
        .function(
            console_method(Self::info, state.clone(), logger.clone()),
            js_string!("info"),
            0,
        )
        .function(
            console_method(Self::log, state.clone(), logger.clone()),
            js_string!("log"),
            0,
        )
        .function(
            console_method(Self::trace, state.clone(), logger.clone()),
            js_string!("trace"),
            0,
        )
        .function(
            console_method(Self::warn, state.clone(), logger.clone()),
            js_string!("warn"),
            0,
        )
        .function(
            console_method_mut(Self::count, state.clone(), logger.clone()),
            js_string!("count"),
            0,
        )
        .function(
            console_method_mut(Self::count_reset, state.clone(), logger.clone()),
            js_string!("countReset"),
            0,
        )
        .function(
            console_method_mut(Self::group, state.clone(), logger.clone()),
            js_string!("group"),
            0,
        )
        .function(
            console_method_mut(Self::group_collapsed, state.clone(), logger.clone()),
            js_string!("groupCollapsed"),
            0,
        )
        .function(
            console_method_mut(Self::group_end, state.clone(), logger.clone()),
            js_string!("groupEnd"),
            0,
        )
        .function(
            console_method_mut(Self::time, state.clone(), logger.clone()),
            js_string!("time"),
            0,
        )
        .function(
            console_method(Self::time_log, state.clone(), logger.clone()),
            js_string!("timeLog"),
            0,
        )
        .function(
            console_method_mut(Self::time_end, state.clone(), logger.clone()),
            js_string!("timeEnd"),
            0,
        )
        .function(
            console_method(Self::dir, state.clone(), logger.clone()),
            js_string!("dir"),
            0,
        )
        .function(
            console_method(Self::dir, state, logger.clone()),
            js_string!("dirxml"),
            0,
        )
        .build()
    }

    /// Initializes the `console` built-in object.
    pub fn init(context: &mut Context) -> JsObject {
        Self::init_with_logger(context, DefaultLogger)
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
        logger: &impl Logger,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let assertion = args.first().map_or(false, JsValue::to_boolean);

        if !assertion {
            let mut args: Vec<JsValue> = args.iter().skip(1).cloned().collect();
            let message = js_string!("Assertion failed");
            if args.is_empty() {
                args.push(JsValue::new(message));
            } else if !args[0].is_string() {
                args.insert(0, JsValue::new(message));
            } else {
                let value = JsString::from(args[0].display().to_string());
                let concat = js_string!(message.as_str(), js_str!(": "), &value);
                args[0] = JsValue::new(concat);
            }

            logger.error(formatter(&args, context)?, &console.state, context)?;
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
        _: &impl Logger,
        _: &mut Context,
    ) -> JsResult<JsValue> {
        console.state.groups.clear();
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
        logger: &impl Logger,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        logger.log(formatter(args, context)?, &console.state, context)?;
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
        logger: &impl Logger,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        logger.error(formatter(args, context)?, &console.state, context)?;
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
        logger: &impl Logger,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        logger.info(formatter(args, context)?, &console.state, context)?;
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
        logger: &impl Logger,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        logger.log(formatter(args, context)?, &console.state, context)?;
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
        logger: &impl Logger,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if !args.is_empty() {
            logger.log(formatter(args, context)?, &console.state, context)?;
        }

        let stack_trace_dump = context
            .stack_trace()
            .map(|frame| frame.code_block().name())
            .collect::<Vec<_>>()
            .into_iter()
            .map(JsString::to_std_string_escaped)
            .collect::<Vec<_>>()
            .join("\n");
        logger.log(stack_trace_dump, &console.state, context)?;

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
        logger: &impl Logger,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        logger.warn(formatter(args, context)?, &console.state, context)?;
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
        logger: &impl Logger,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let label = match args.first() {
            Some(value) => value.to_string(context)?,
            None => "default".into(),
        };

        let msg = format!("count {}:", label.to_std_string_escaped());
        let c = console.state.count_map.entry(label).or_insert(0);
        *c += 1;

        logger.info(format!("{msg} {c}"), &console.state, context)?;
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
        logger: &impl Logger,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let label = match args.first() {
            Some(value) => value.to_string(context)?,
            None => "default".into(),
        };

        console.state.count_map.remove(&label);

        logger.warn(
            format!("countReset {}", label.to_std_string_escaped()),
            &console.state,
            context,
        )?;

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
        logger: &impl Logger,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let label = match args.first() {
            Some(value) => value.to_string(context)?,
            None => "default".into(),
        };

        if let Entry::Vacant(e) = console.state.timer_map.entry(label.clone()) {
            let time = Self::system_time_in_ms();
            e.insert(time);
        } else {
            logger.warn(
                format!("Timer '{}' already exist", label.to_std_string_escaped()),
                &console.state,
                context,
            )?;
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
        logger: &impl Logger,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let label = match args.first() {
            Some(value) => value.to_string(context)?,
            None => "default".into(),
        };

        if let Some(t) = console.state.timer_map.get(&label) {
            let time = Self::system_time_in_ms();
            let mut concat = format!("{}: {} ms", label.to_std_string_escaped(), time - t);
            for msg in args.iter().skip(1) {
                concat = concat + " " + &msg.display().to_string();
            }
            logger.log(concat, &console.state, context)?;
        } else {
            logger.warn(
                format!("Timer '{}' doesn't exist", label.to_std_string_escaped()),
                &console.state,
                context,
            )?;
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
    fn time_end(
        _: &JsValue,
        args: &[JsValue],
        console: &mut Self,
        logger: &impl Logger,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let label = match args.first() {
            Some(value) => value.to_string(context)?,
            None => "default".into(),
        };

        if let Some(t) = console.state.timer_map.remove(&label) {
            let time = Self::system_time_in_ms();
            logger.info(
                format!(
                    "{}: {} ms - timer removed",
                    label.to_std_string_escaped(),
                    time - t
                ),
                &console.state,
                context,
            )?;
        } else {
            logger.warn(
                format!("Timer '{}' doesn't exist", label.to_std_string_escaped()),
                &console.state,
                context,
            )?;
        };

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
        logger: &impl Logger,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let group_label = formatter(args, context)?;

        logger.info(format!("group: {group_label}"), &console.state, context)?;
        console.state.groups.push(group_label);

        Ok(JsValue::undefined())
    }

    /// `console.groupCollapsed(...data)`
    ///
    /// Adds new group collapsed with name from formatted data to stack.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///  - [WHATWG `console` specification][spec]
    ///
    /// [spec]: https://console.spec.whatwg.org/#groupcollapsed
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/console/groupcollapsed_static
    fn group_collapsed(
        _: &JsValue,
        args: &[JsValue],
        console: &mut Self,
        logger: &impl Logger,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Console::group(&JsValue::Undefined, args, console, logger, context)
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
        _: &impl Logger,
        _: &mut Context,
    ) -> JsResult<JsValue> {
        console.state.groups.pop();

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
        logger: &impl Logger,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        logger.info(
            args.get_or_undefined(0).display_obj(true),
            &console.state,
            context,
        )?;
        Ok(JsValue::undefined())
    }
}
