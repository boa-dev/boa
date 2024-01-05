//! Boa's implementation of ECMAScript's `Date` object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-date-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::{
        intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
        HostHooks,
    },
    error::JsNativeError,
    js_string,
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    property::Attribute,
    realm::Realm,
    string::{common::StaticJsStrings, utf16},
    symbol::JsSymbol,
    value::{IntegerOrNan, JsValue, PreferredType},
    Context, JsArgs, JsData, JsError, JsResult, JsString,
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;
use chrono::{Datelike, NaiveDateTime, TimeZone, Timelike, Utc};
use utils::{
    make_date, make_day, make_time, parse_date, replace_params, time_clip, DateParameters,
};

pub(crate) mod utils;

#[cfg(test)]
mod tests;

/// Extracts `Some` from an `Option<T>` or returns `NaN` if the object contains `None`.
macro_rules! some_or_nan {
    ($v:expr) => {
        match $v {
            Some(dt) => dt,
            None => return Ok(JsValue::from(f64::NAN)),
        }
    };
}

/// Gets a mutable reference to the inner `Date` object of `val`, or returns
/// a `TypeError` if `val` is not a `Date` object.
macro_rules! get_mut_date {
    ($val:expr) => {
        $val.as_object()
            .and_then(JsObject::downcast_mut::<Date>)
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Date"))?
    };
}

/// Abstract operation [`thisTimeValue`][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-thistimevalue
pub(super) fn this_time_value(value: &JsValue) -> JsResult<Option<i64>> {
    Ok(value
        .as_object()
        .and_then(|obj| obj.downcast_ref::<Date>().as_deref().copied())
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Date"))?
        .0)
}

/// The internal representation of a `Date` object.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Trace, Finalize, JsData)]
#[boa_gc(empty_trace)]
pub struct Date(Option<i64>);

impl Date {
    /// Creates a new `Date`.
    pub(crate) const fn new(dt: Option<i64>) -> Self {
        Self(dt)
    }

    /// Creates a new `Date` from the current UTC time of the host.
    pub(crate) fn utc_now(hooks: &dyn HostHooks) -> Self {
        let dt = hooks.utc_now();
        Self(Some(dt.timestamp_millis()))
    }

    /// Converts the `Date` into a `JsValue`, mapping `None` to `NaN` and `Some(datetime)` to
    /// `JsValue::from(datetime.timestamp_millis())`.
    fn as_value(&self) -> JsValue {
        self.0.map_or_else(|| f64::NAN.into(), Into::into)
    }
}

impl IntrinsicObject for Date {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        let to_utc_string = BuiltInBuilder::callable(realm, Self::to_utc_string)
            .name(js_string!("toUTCString"))
            .length(0)
            .build();

        let to_primitive = BuiltInBuilder::callable(realm, Self::to_primitive)
            .name(js_string!("[Symbol.toPrimitive]"))
            .length(1)
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_method(Self::now, js_string!("now"), 0)
            .static_method(Self::parse, js_string!("parse"), 1)
            .static_method(Self::utc, js_string!("UTC"), 7)
            .method(Self::get_date::<true>, js_string!("getDate"), 0)
            .method(Self::get_day::<true>, js_string!("getDay"), 0)
            .method(Self::get_full_year::<true>, js_string!("getFullYear"), 0)
            .method(Self::get_hours::<true>, js_string!("getHours"), 0)
            .method(
                Self::get_milliseconds::<true>,
                js_string!("getMilliseconds"),
                0,
            )
            .method(Self::get_minutes::<true>, js_string!("getMinutes"), 0)
            .method(Self::get_month::<true>, js_string!("getMonth"), 0)
            .method(Self::get_seconds::<true>, js_string!("getSeconds"), 0)
            .method(Self::get_time, js_string!("getTime"), 0)
            .method(
                Self::get_timezone_offset,
                js_string!("getTimezoneOffset"),
                0,
            )
            .method(Self::get_date::<false>, js_string!("getUTCDate"), 0)
            .method(Self::get_day::<false>, js_string!("getUTCDay"), 0)
            .method(
                Self::get_full_year::<false>,
                js_string!("getUTCFullYear"),
                0,
            )
            .method(Self::get_hours::<false>, js_string!("getUTCHours"), 0)
            .method(
                Self::get_milliseconds::<false>,
                js_string!("getUTCMilliseconds"),
                0,
            )
            .method(Self::get_minutes::<false>, js_string!("getUTCMinutes"), 0)
            .method(Self::get_month::<false>, js_string!("getUTCMonth"), 0)
            .method(Self::get_seconds::<false>, js_string!("getUTCSeconds"), 0)
            .method(Self::get_year, js_string!("getYear"), 0)
            .method(Self::set_date::<true>, js_string!("setDate"), 1)
            .method(Self::set_full_year::<true>, js_string!("setFullYear"), 3)
            .method(Self::set_hours::<true>, js_string!("setHours"), 4)
            .method(
                Self::set_milliseconds::<true>,
                js_string!("setMilliseconds"),
                1,
            )
            .method(Self::set_minutes::<true>, js_string!("setMinutes"), 3)
            .method(Self::set_month::<true>, js_string!("setMonth"), 2)
            .method(Self::set_seconds::<true>, js_string!("setSeconds"), 2)
            .method(Self::set_time, js_string!("setTime"), 1)
            .method(Self::set_date::<false>, js_string!("setUTCDate"), 1)
            .method(
                Self::set_full_year::<false>,
                js_string!("setUTCFullYear"),
                3,
            )
            .method(Self::set_hours::<false>, js_string!("setUTCHours"), 4)
            .method(
                Self::set_milliseconds::<false>,
                js_string!("setUTCMilliseconds"),
                1,
            )
            .method(Self::set_minutes::<false>, js_string!("setUTCMinutes"), 3)
            .method(Self::set_month::<false>, js_string!("setUTCMonth"), 2)
            .method(Self::set_seconds::<false>, js_string!("setUTCSeconds"), 2)
            .method(Self::set_year, js_string!("setYear"), 1)
            .method(Self::to_date_string, js_string!("toDateString"), 0)
            .method(Self::to_iso_string, js_string!("toISOString"), 0)
            .method(Self::to_json, js_string!("toJSON"), 1)
            .method(
                Self::to_locale_date_string,
                js_string!("toLocaleDateString"),
                0,
            )
            .method(Self::to_locale_string, js_string!("toLocaleString"), 0)
            .method(
                Self::to_locale_time_string,
                js_string!("toLocaleTimeString"),
                0,
            )
            .method(Self::to_string, js_string!("toString"), 0)
            .method(Self::to_time_string, js_string!("toTimeString"), 0)
            .method(Self::value_of, js_string!("valueOf"), 0)
            .property(
                js_string!("toGMTString"),
                to_utc_string.clone(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("toUTCString"),
                to_utc_string,
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                JsSymbol::to_primitive(),
                to_primitive,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Date {
    const NAME: JsString = StaticJsStrings::DATE;
}

impl BuiltInConstructor for Date {
    const LENGTH: usize = 7;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::date;

    /// [`Date ( ...values )`][spec]
    ///
    /// - When called as a function, returns a string displaying the current time in the UTC timezone.
    /// - When called as a constructor, it returns a new `Date` object from the provided arguments.
    /// The [MDN documentation][mdn] has a more extensive explanation on the usages and return
    /// values for all possible arguments.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date-constructor
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/Date
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, then
        if new_target.is_undefined() {
            // a. Let now be the time value (UTC) identifying the current time.
            // b. Return ToDateString(now).
            return Ok(JsValue::new(js_string!(context
                .host_hooks()
                .local_from_utc(context.host_hooks().utc_now())
                .format("%a %b %d %Y %H:%M:%S GMT%:z")
                .to_string())));
        }
        // 2. Let numberOfArgs be the number of elements in values.
        let dv = match args {
            // 3. If numberOfArgs = 0, then
            [] => {
                // a. Let dv be the time value (UTC) identifying the current time.
                Self::utc_now(context.host_hooks())
            }
            // 4. Else if numberOfArgs = 1, then
            // a. Let value be values[0].
            [value] => match value
                .as_object()
                .and_then(|obj| obj.downcast_ref::<Self>().as_deref().copied())
            {
                // b. If value is an Object and value has a [[DateValue]] internal slot, then
                Some(dt) => {
                    // i. Let tv be ! thisTimeValue(value).
                    dt
                }
                // c. Else,
                None => {
                    // i. Let v be ? ToPrimitive(value).
                    match value.to_primitive(context, PreferredType::Default)? {
                        // ii. If v is a String, then
                        JsValue::String(ref str) => {
                            // 1. Assert: The next step never returns an abrupt completion because v is a String.
                            // 2. Let tv be the result of parsing v as a date, in exactly the same manner as for the
                            // parse method (21.4.3.2).
                            Self::new(parse_date(str, context.host_hooks()))
                        }
                        // iii. Else,
                        v => {
                            // Directly convert to integer
                            // 1. Let tv be ? ToNumber(v).

                            let dt = v
                                .to_integer_or_nan(context)?
                                .as_integer()
                                // d. Let dv be TimeClip(tv).
                                .and_then(time_clip);
                            Self(dt)
                        }
                    }
                }
            },
            // 5. Else,
            _ => {
                // Separating this into its own function to simplify the logic.

                let dt = Self::construct_date(args, context)?
                    .and_then(|dt| context.host_hooks().local_from_naive_local(dt).earliest());

                Self(dt.map(|dt| dt.timestamp_millis()))
            }
        };

        // 6. Let O be ? OrdinaryCreateFromConstructor(NewTarget, "%Date.prototype%", « [[DateValue]] »).
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::date, context)?;

        // 7. Set O.[[DateValue]] to dv.
        let obj =
            JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), prototype, dv);

        // 8. Return O.
        Ok(obj.into())
    }
}

impl Date {
    /// Gets the timestamp from a list of component values.
    fn construct_date(
        values: &[JsValue],
        context: &mut Context,
    ) -> JsResult<Option<NaiveDateTime>> {
        // 1. Let y be ? ToNumber(year).
        let Some(mut year) = values
            .get_or_undefined(0)
            .to_integer_or_nan(context)?
            .as_integer()
        else {
            return Ok(None);
        };

        // 2. If month is present, let m be ? ToNumber(month); else let m be +0𝔽.
        let Some(month) = values.get(1).map_or(Ok(Some(0)), |value| {
            value
                .to_integer_or_nan(context)
                .map(IntegerOrNan::as_integer)
        })?
        else {
            return Ok(None);
        };

        // 3. If date is present, let dt be ? ToNumber(date); else let dt be 1𝔽.
        let Some(date) = values.get(2).map_or(Ok(Some(1)), |value| {
            value
                .to_integer_or_nan(context)
                .map(IntegerOrNan::as_integer)
        })?
        else {
            return Ok(None);
        };

        // 4. If hours is present, let h be ? ToNumber(hours); else let h be +0𝔽.
        let Some(hour) = values.get(3).map_or(Ok(Some(0)), |value| {
            value
                .to_integer_or_nan(context)
                .map(IntegerOrNan::as_integer)
        })?
        else {
            return Ok(None);
        };

        // 5. If minutes is present, let min be ? ToNumber(minutes); else let min be +0𝔽.
        let Some(min) = values.get(4).map_or(Ok(Some(0)), |value| {
            value
                .to_integer_or_nan(context)
                .map(IntegerOrNan::as_integer)
        })?
        else {
            return Ok(None);
        };

        // 6. If seconds is present, let s be ? ToNumber(seconds); else let s be +0𝔽.
        let Some(sec) = values.get(5).map_or(Ok(Some(0)), |value| {
            value
                .to_integer_or_nan(context)
                .map(IntegerOrNan::as_integer)
        })?
        else {
            return Ok(None);
        };

        // 7. If ms is present, let milli be ? ToNumber(ms); else let milli be +0𝔽.
        let Some(ms) = values.get(6).map_or(Ok(Some(0)), |value| {
            value
                .to_integer_or_nan(context)
                .map(IntegerOrNan::as_integer)
        })?
        else {
            return Ok(None);
        };

        // 8. If y is NaN, let yr be NaN.
        // 9. Else,
        //     a. Let yi be ! ToIntegerOrInfinity(y).
        //     b. If 0 ≤ yi ≤ 99, let yr be 1900𝔽 + 𝔽(yi); otherwise, let yr be y.
        if (0..=99).contains(&year) {
            year += 1900;
        }

        // 10. Return TimeClip(MakeDate(MakeDay(yr, m, dt), MakeTime(h, min, s, milli))).
        // PLEASE RUST TEAM GIVE US TRY BLOCKS ;-;
        let timestamp = (move || {
            let day = make_day(year, month, date)?;
            let time = make_time(hour, min, sec, ms)?;
            make_date(day, time)
        })();

        Ok(timestamp
            .and_then(time_clip)
            .and_then(NaiveDateTime::from_timestamp_millis))
    }

    /// `Date.now()`
    ///
    /// The static `Date.now()` method returns the number of milliseconds elapsed since January 1, 1970 00:00:00 UTC.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.now
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/now
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn now(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::new(
            context.host_hooks().utc_now().timestamp_millis(),
        ))
    }

    /// `Date.parse()`
    ///
    /// The `Date.parse()` method parses a string representation of a date, and returns the number of milliseconds since
    /// January 1, 1970, 00:00:00 UTC or `NaN` if the string is unrecognized or, in some cases, contains illegal date
    /// values.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.parse
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/parse
    pub(crate) fn parse(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let date = args.get_or_undefined(0).to_string(context)?;
        Ok(parse_date(&date, context.host_hooks()).map_or(JsValue::from(f64::NAN), JsValue::from))
    }

    /// `Date.UTC()`
    ///
    /// The `Date.UTC()` method accepts parameters similar to the `Date` constructor, but treats them as UTC.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.utc
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/UTC
    pub(crate) fn utc(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let t = some_or_nan!(Self::construct_date(args, context)?);

        Ok(JsValue::from(t.timestamp_millis()))
    }

    /// [`Date.prototype.getDate ( )`][local] and
    /// [`Date.prototype.getUTCDate ( )`][utc].
    ///
    /// The `getDate()` method returns the day of the month for the specified date.
    ///
    /// [local]: https://tc39.es/ecma262/#sec-date.prototype.getdate
    /// [utc]: https://tc39.es/ecma262/#sec-date.prototype.getutcdate
    pub(crate) fn get_date<const LOCAL: bool>(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let t = this_time_value(this)?;

        // 2. If t is NaN, return NaN.
        let t = some_or_nan!(t
            .and_then(NaiveDateTime::from_timestamp_millis)
            .map(|dt| if LOCAL {
                context.host_hooks().local_from_utc(dt).naive_local()
            } else {
                dt
            }));

        // 3. Return DateFromTime(LocalTime(t)).
        Ok(JsValue::new(t.day()))
    }

    /// [`Date.prototype.getDay ( )`][local] and
    /// [`Date.prototype.getUTCDay ( )`][utc].
    ///
    /// The `getDay()` method returns the day of the week for the specified date, where 0 represents
    /// Sunday.
    ///
    /// [local]: https://tc39.es/ecma262/#sec-date.prototype.getday
    /// [utc]: https://tc39.es/ecma262/#sec-date.prototype.getutcday
    pub(crate) fn get_day<const LOCAL: bool>(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let t = this_time_value(this)?;

        // 2. If t is NaN, return NaN.
        let t = some_or_nan!(t
            .and_then(NaiveDateTime::from_timestamp_millis)
            .map(|dt| if LOCAL {
                context.host_hooks().local_from_utc(dt).naive_local()
            } else {
                dt
            }));

        // 3. Return WeekDay(LocalTime(t)).
        Ok(JsValue::new(t.weekday().num_days_from_sunday()))
    }

    /// [`Date.prototype.getYear()`][spec].
    ///
    /// The `getYear()` method returns the year in the specified date according to local time.
    /// Because `getYear()` does not return full years ("year 2000 problem"), it is no longer used
    /// and has been replaced by the `getFullYear()` method.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.getyear
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getYear
    pub(crate) fn get_year(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let t = this_time_value(this)?;

        // 2. If t is NaN, return NaN.
        let t = some_or_nan!(t.and_then(NaiveDateTime::from_timestamp_millis));

        // 3. Return YearFromTime(LocalTime(t)) - 1900𝔽.
        let local = context.host_hooks().local_from_utc(t);
        Ok(JsValue::from(local.year() - 1900))
    }

    /// [`Date.prototype.getFullYear ( )`][local] and
    /// [`Date.prototype.getUTCFullYear ( )`][utc].
    ///
    /// The `getFullYear()` method returns the year of the specified date.
    ///
    /// [local]: https://tc39.es/ecma262/#sec-date.prototype.getfullyear
    /// [utc]: https://tc39.es/ecma262/#sec-date.prototype.getutcfullyear
    pub(crate) fn get_full_year<const LOCAL: bool>(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let t = this_time_value(this)?;

        // 2. If t is NaN, return NaN.
        let t = some_or_nan!(t
            .and_then(NaiveDateTime::from_timestamp_millis)
            .map(|dt| if LOCAL {
                context.host_hooks().local_from_utc(dt).naive_local()
            } else {
                dt
            }));

        // 3. Return YearFromTime(LocalTime(t)).
        Ok(JsValue::new(t.year()))
    }

    /// [`Date.prototype.getHours ( )`][local] and
    /// [`Date.prototype.getUTCHours ( )`][utc].
    ///
    /// The `getHours()` method returns the hour for the specified date.
    ///
    /// [local]: https://tc39.es/ecma262/#sec-date.prototype.gethours
    /// [utc]: https://tc39.es/ecma262/#sec-date.prototype.getutchours
    pub(crate) fn get_hours<const LOCAL: bool>(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let t = this_time_value(this)?;

        // 2. If t is NaN, return NaN.
        let t = some_or_nan!(t
            .and_then(NaiveDateTime::from_timestamp_millis)
            .map(|dt| if LOCAL {
                context.host_hooks().local_from_utc(dt).naive_local()
            } else {
                dt
            }));

        // 3. Return HourFromTime(LocalTime(t)).
        Ok(JsValue::new(t.hour()))
    }

    /// [`Date.prototype.getMilliseconds ( )`][local] and
    /// [`Date.prototype.getUTCMilliseconds ( )`][utc].
    ///
    /// The `getMilliseconds()` method returns the milliseconds in the specified date.
    ///
    /// [local]: https://tc39.es/ecma262/#sec-date.prototype.getmilliseconds
    /// [utc]: https://tc39.es/ecma262/#sec-date.prototype.getutcmilliseconds
    pub(crate) fn get_milliseconds<const LOCAL: bool>(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let t = this_time_value(this)?;

        // 2. If t is NaN, return NaN.
        let t = some_or_nan!(t
            .and_then(NaiveDateTime::from_timestamp_millis)
            .map(|dt| if LOCAL {
                context.host_hooks().local_from_utc(dt).naive_local()
            } else {
                dt
            }));

        // 3. Return msFromTime(LocalTime(t)).
        Ok(JsValue::new(t.timestamp_subsec_millis()))
    }

    /// [`Date.prototype.getMinutes ( )`][local] and
    /// [`Date.prototype.getUTCMinutes ( )`][utc].
    ///
    /// The `getMinutes()` method returns the minutes in the specified date.
    ///
    /// [local]: https://tc39.es/ecma262/#sec-date.prototype.getminutes
    /// [utc]: https://tc39.es/ecma262/#sec-date.prototype.getutcminutes
    pub(crate) fn get_minutes<const LOCAL: bool>(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let t = this_time_value(this)?;

        // 2. If t is NaN, return NaN.
        let t = some_or_nan!(t
            .and_then(NaiveDateTime::from_timestamp_millis)
            .map(|dt| if LOCAL {
                context.host_hooks().local_from_utc(dt).naive_local()
            } else {
                dt
            }));

        // 3. Return MinFromTime(LocalTime(t)).
        Ok(JsValue::new(t.minute()))
    }

    /// [`Date.prototype.getMonth ( )`][local] and
    /// [`Date.prototype.getUTCMonth ( )`][utc].
    ///
    /// The `getMonth()` method returns the month in the specified date, as a zero-based value
    /// (where zero indicates the first month of the year).
    ///
    /// [local]: https://tc39.es/ecma262/#sec-date.prototype.getmonth
    /// [utc]: https://tc39.es/ecma262/#sec-date.prototype.getutcmonth
    pub(crate) fn get_month<const LOCAL: bool>(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let t = this_time_value(this)?;

        // 2. If t is NaN, return NaN.
        let t = some_or_nan!(t
            .and_then(NaiveDateTime::from_timestamp_millis)
            .map(|dt| if LOCAL {
                context.host_hooks().local_from_utc(dt).naive_local()
            } else {
                dt
            }));

        // 3. Return MonthFromTime(LocalTime(t)).
        Ok(JsValue::new(t.month0()))
    }

    /// [`Date.prototype.getSeconds ( )`][local] and
    /// [`Date.prototype.getUTCSeconds ( )`][utc].
    ///
    /// The `getSeconds()` method returns the seconds in the specified date.
    ///
    /// [local]: https://tc39.es/ecma262/#sec-date.prototype.getseconds
    /// [utc]: https://tc39.es/ecma262/#sec-date.prototype.getutcseconds
    pub(crate) fn get_seconds<const LOCAL: bool>(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let t = this_time_value(this)?;

        // 2. If t is NaN, return NaN.
        let t = some_or_nan!(t
            .and_then(NaiveDateTime::from_timestamp_millis)
            .map(|dt| if LOCAL {
                context.host_hooks().local_from_utc(dt).naive_local()
            } else {
                dt
            }));

        // 3. Return SecFromTime(LocalTime(t))
        Ok(JsValue::new(t.second()))
    }

    /// `Date.prototype.getTime()`.
    ///
    /// The `getTime()` method returns the number of milliseconds since the Unix Epoch.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.gettime
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getTime
    pub(crate) fn get_time(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Return ? thisTimeValue(this value).
        Ok(this_time_value(this)?.map_or(JsValue::nan(), JsValue::from))
    }

    /// `Date.prototype.getTimeZoneOffset()`.
    ///
    /// The `getTimezoneOffset()` method returns the time zone difference, in minutes, from current locale (host system
    /// settings) to UTC.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.gettimezoneoffset
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getTimezoneOffset
    pub(crate) fn get_timezone_offset(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        // 2. If t is NaN, return NaN.
        let t = some_or_nan!(this_time_value(this)?.and_then(NaiveDateTime::from_timestamp_millis));

        // 3. Return (t - LocalTime(t)) / msPerMinute.
        Ok(JsValue::from(
            -context
                .host_hooks()
                .local_from_utc(t)
                .offset()
                .local_minus_utc()
                / 60,
        ))
    }

    /// [`Date.prototype.setDate ( date )`][local] and
    /// [`Date.prototype.setUTCDate ( date )`][utc].
    ///
    /// The `setDate()` method sets the day of the `Date` object relative to the beginning of the
    /// currently set month.
    ///
    /// [local]: https://tc39.es/ecma262/#sec-date.prototype.setdate
    /// [utc]: https://tc39.es/ecma262/#sec-date.prototype.setutcdate
    pub(crate) fn set_date<const LOCAL: bool>(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be LocalTime(? thisTimeValue(this value)).
        let mut t = get_mut_date!(this);

        // 2. Let dt be ? ToNumber(date).
        let date = args.get_or_undefined(0).to_integer_or_nan(context)?;

        // 3. If t is NaN, return NaN.
        let datetime = some_or_nan!(t.0);

        // 4. Set t to LocalTime(t).
        // 5. Let newDate be MakeDate(MakeDay(YearFromTime(t), MonthFromTime(t), dt), TimeWithinDay(t)).
        // 6. Let u be TimeClip(UTC(newDate)).
        let datetime = replace_params::<LOCAL>(
            datetime,
            DateParameters {
                date: Some(date),
                ..Default::default()
            },
            context.host_hooks(),
        );

        // 7. Set the [[DateValue]] internal slot of this Date object to u.
        *t = Self::new(datetime);

        // 8. Return u.
        Ok(t.as_value())
    }

    /// [`Date.prototype.setFullYear ( year [ , month [ , date ] ] )`][local] and
    /// [Date.prototype.setUTCFullYear ( year [ , month [ , date ] ] )][utc].
    ///
    /// The `setFullYear()` method sets the full year for a specified date and returns the new
    /// timestamp.
    ///
    /// [local]: https://tc39.es/ecma262/#sec-date.prototype.setfullyear
    /// [utc]: https://tc39.es/ecma262/#sec-date.prototype.setutcfullyear
    pub(crate) fn set_full_year<const LOCAL: bool>(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let mut t = get_mut_date!(this);

        // 2. If t is NaN, set t to +0𝔽; otherwise, set t to LocalTime(t).
        let datetime =
            t.0.and_then(NaiveDateTime::from_timestamp_millis)
                .or_else(|| {
                    if LOCAL {
                        context
                            .host_hooks()
                            .local_from_naive_local(NaiveDateTime::default())
                            .earliest()
                            .map(|dt| dt.naive_utc())
                    } else {
                        Some(NaiveDateTime::default())
                    }
                });
        let datetime = some_or_nan!(datetime);

        // 3. Let y be ? ToNumber(year).
        let year = args.get_or_undefined(0).to_integer_or_nan(context)?;

        // 4. If month is not present, let m be MonthFromTime(t); otherwise, let m be ? ToNumber(month).
        let month = args
            .get(1)
            .map(|v| v.to_integer_or_nan(context))
            .transpose()?;

        // 5. If date is not present, let dt be DateFromTime(t); otherwise, let dt be ? ToNumber(date).
        let date = args
            .get(2)
            .map(|v| v.to_integer_or_nan(context))
            .transpose()?;

        // 6. Let newDate be MakeDate(MakeDay(y, m, dt), TimeWithinDay(t)).
        // 7. Let u be TimeClip(UTC(newDate)).
        let datetime = replace_params::<LOCAL>(
            datetime.timestamp_millis(),
            DateParameters {
                year: Some(year),
                month,
                date,
                ..Default::default()
            },
            context.host_hooks(),
        );

        // 8. Set the [[DateValue]] internal slot of this Date object to u.
        *t = Self::new(datetime);

        // 9. Return u.
        Ok(t.as_value())
    }

    /// [`Date.prototype.setHours ( hour [ , min [ , sec [ , ms ] ] ] )`][local] and
    /// [`Date.prototype.setUTCHours ( hour [ , min [ , sec [ , ms ] ] ] )`][utc].
    ///
    /// The `setHours()` method sets the hours for a specified date, and returns the number
    /// of milliseconds since January 1, 1970 00:00:00 UTC until the time represented by the
    /// updated `Date` instance.
    ///
    /// [local]: https://tc39.es/ecma262/#sec-date.prototype.sethours
    /// [utc]: https://tc39.es/ecma262/#sec-date.prototype.setutchours
    pub(crate) fn set_hours<const LOCAL: bool>(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let mut t = get_mut_date!(this);

        // 2. Let h be ? ToNumber(hour).
        let hour = args.get_or_undefined(0).to_integer_or_nan(context)?;

        // 3. If min is present, let m be ? ToNumber(min).
        let minute = args
            .get(1)
            .map(|v| v.to_integer_or_nan(context))
            .transpose()?;

        // 4. If sec is present, let s be ? ToNumber(sec).
        let second = args
            .get(2)
            .map(|v| v.to_integer_or_nan(context))
            .transpose()?;

        // 5. If ms is present, let milli be ? ToNumber(ms).
        let millisecond = args
            .get(3)
            .map(|v| v.to_integer_or_nan(context))
            .transpose()?;

        // 6. If t is NaN, return NaN.
        let datetime = some_or_nan!(t.0);

        // 7. Set t to LocalTime(t).
        // 8. If min is not present, let m be MinFromTime(t).
        // 9. If sec is not present, let s be SecFromTime(t).
        // 10. If ms is not present, let milli be msFromTime(t).
        // 11. Let date be MakeDate(Day(t), MakeTime(h, m, s, milli)).
        // 12. Let u be TimeClip(UTC(date)).
        let datetime = replace_params::<LOCAL>(
            datetime,
            DateParameters {
                hour: Some(hour),
                minute,
                second,
                millisecond,
                ..Default::default()
            },
            context.host_hooks(),
        );

        // 13. Set the [[DateValue]] internal slot of this Date object to u.
        *t = Self::new(datetime);

        // 14. Return u.
        Ok(t.as_value())
    }

    /// [`Date.prototype.setMilliseconds ( ms )`[local] and
    /// [`Date.prototype.setUTCMilliseconds ( ms )`][utc].
    ///
    /// The `setMilliseconds()` method sets the milliseconds for a specified date according to local time.
    ///
    /// [local]: https://tc39.es/ecma262/#sec-date.prototype.setmilliseconds
    /// [utc]: https://tc39.es/ecma262/#sec-date.prototype.setutcmilliseconds
    pub(crate) fn set_milliseconds<const LOCAL: bool>(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        // 1. Let t be LocalTime(? thisTimeValue(this value)).
        let mut t = get_mut_date!(this);

        // 2. Set ms to ? ToNumber(ms).
        let ms = args.get_or_undefined(0).to_integer_or_nan(context)?;

        // 3. If t is NaN, return NaN.
        let datetime = some_or_nan!(t.0);

        // 4. Set t to LocalTime(t).
        // 5. Let time be MakeTime(HourFromTime(t), MinFromTime(t), SecFromTime(t), ms).
        // 6. Let u be TimeClip(UTC(MakeDate(Day(t), time))).
        let datetime = replace_params::<LOCAL>(
            datetime,
            DateParameters {
                millisecond: Some(ms),
                ..Default::default()
            },
            context.host_hooks(),
        );

        // 7. Set the [[DateValue]] internal slot of this Date object to u.
        *t = Self::new(datetime);

        // 8. Return u.
        Ok(t.as_value())
    }

    /// [`Date.prototype.setMinutes ( min [ , sec [ , ms ] ] )`][local] and
    /// [`Date.prototype.setUTCMinutes ( min [ , sec [ , ms ] ] )`][utc].
    ///
    /// The `setMinutes()` method sets the minutes for a specified date.
    ///
    /// [local]: https://tc39.es/ecma262/#sec-date.prototype.setminutes
    /// [utc]: https://tc39.es/ecma262/#sec-date.prototype.setutcminutes
    pub(crate) fn set_minutes<const LOCAL: bool>(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let mut t = get_mut_date!(this);

        // 2. Let m be ? ToNumber(min).
        let minute = args.get_or_undefined(0).to_integer_or_nan(context)?;

        // 3. If sec is present, let s be ? ToNumber(sec).
        let second = args
            .get(1)
            .map(|v| v.to_integer_or_nan(context))
            .transpose()?;

        // 4. If ms is present, let milli be ? ToNumber(ms).
        let millisecond = args
            .get(2)
            .map(|v| v.to_integer_or_nan(context))
            .transpose()?;

        // 5. If t is NaN, return NaN.
        let datetime = some_or_nan!(t.0);

        // 6. Set t to LocalTime(t).
        // 7. If sec is not present, let s be SecFromTime(t).
        // 8. If ms is not present, let milli be msFromTime(t).
        // 9. Let date be MakeDate(Day(t), MakeTime(HourFromTime(t), m, s, milli)).
        // 10. Let u be TimeClip(UTC(date)).
        let datetime = replace_params::<LOCAL>(
            datetime,
            DateParameters {
                minute: Some(minute),
                second,
                millisecond,
                ..Default::default()
            },
            context.host_hooks(),
        );

        // 11. Set the [[DateValue]] internal slot of this Date object to u.
        *t = Self::new(datetime);

        // 12. Return u.
        Ok(t.as_value())
    }

    /// [`Date.prototype.setMonth ( month [ , date ] )`][local] and
    /// [`Date.prototype.setUTCMonth ( month [ , date ] )`][utc].
    ///
    /// The `setMonth()` method sets the month for a specified date according to the currently set
    /// year.
    ///
    /// [local]: https://tc39.es/ecma262/#sec-date.prototype.setmonth
    /// [utc]: https://tc39.es/ecma262/#sec-date.prototype.setutcmonth
    pub(crate) fn set_month<const LOCAL: bool>(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let mut t = get_mut_date!(this);

        // 2. Let m be ? ToNumber(month).
        let month = args.get_or_undefined(0).to_integer_or_nan(context)?;

        // 3. If date is present, let dt be ? ToNumber(date).
        let date = args
            .get(1)
            .map(|v| v.to_integer_or_nan(context))
            .transpose()?;

        // 4. If t is NaN, return NaN.
        let datetime = some_or_nan!(t.0);

        // 5. Set t to LocalTime(t).
        // 6. If date is not present, let dt be DateFromTime(t).
        // 7. Let newDate be MakeDate(MakeDay(YearFromTime(t), m, dt), TimeWithinDay(t)).
        // 8. Let u be TimeClip(UTC(newDate)).
        let datetime = replace_params::<LOCAL>(
            datetime,
            DateParameters {
                month: Some(month),
                date,
                ..Default::default()
            },
            context.host_hooks(),
        );

        // 9. Set the [[DateValue]] internal slot of this Date object to u.
        *t = Self::new(datetime);

        // 10. Return u.
        Ok(t.as_value())
    }

    /// [`Date.prototype.setSeconds ( sec [ , ms ] )`[local] and
    /// [`Date.prototype.setUTCSeconds ( sec [ , ms ] )`][utc].
    ///
    /// The `setSeconds()` method sets the seconds for a specified date.
    ///
    /// [local]: https://tc39.es/ecma262/#sec-date.prototype.setseconds
    /// [utc]: https://tc39.es/ecma262/#sec-date.prototype.setutcseconds
    pub(crate) fn set_seconds<const LOCAL: bool>(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let mut t = get_mut_date!(this);

        // 2. Let s be ? ToNumber(sec).
        let second = args.get_or_undefined(0).to_integer_or_nan(context)?;

        // 3. If ms is present, let milli be ? ToNumber(ms).
        let millisecond = args
            .get(1)
            .map(|v| v.to_integer_or_nan(context))
            .transpose()?;

        // 4. If t is NaN, return NaN.
        let datetime = some_or_nan!(t.0);

        // 5. Set t to LocalTime(t).
        // 6. If ms is not present, let milli be msFromTime(t).
        // 7. Let date be MakeDate(Day(t), MakeTime(HourFromTime(t), MinFromTime(t), s, milli)).
        // 8. Let u be TimeClip(UTC(date)).
        let datetime = replace_params::<LOCAL>(
            datetime,
            DateParameters {
                second: Some(second),
                millisecond,
                ..Default::default()
            },
            context.host_hooks(),
        );

        // 9. Set the [[DateValue]] internal slot of this Date object to u.
        *t = Self::new(datetime);

        // 10. Return u.
        Ok(t.as_value())
    }

    /// [`Date.prototype.setYear()`][spec].
    ///
    /// The `setYear()` method sets the year for a specified date according to local time.
    ///
    /// # Note
    ///
    /// The [`Self::set_full_year`] method is preferred for nearly all purposes, because it avoids
    /// the “year 2000 problem.”
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setYear
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.setyear
    pub(crate) fn set_year(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let mut t = get_mut_date!(this);

        // 2. Let y be ? ToNumber(year).
        // 5. Let yi be ! ToIntegerOrInfinity(y).
        let year = args.get_or_undefined(0).to_integer_or_nan(context)?;

        // 3. If t is NaN, set t to +0𝔽; otherwise, set t to LocalTime(t).
        let datetime =
            t.0.and_then(NaiveDateTime::from_timestamp_millis)
                .or_else(|| {
                    context
                        .host_hooks()
                        .local_from_naive_local(NaiveDateTime::default())
                        .earliest()
                        .map(|dt| dt.naive_utc())
                });
        let datetime = some_or_nan!(datetime);

        // 4. If y is NaN, then
        let Some(mut year) = year.as_integer() else {
            // a. Set the [[DateValue]] internal slot of this Date object to NaN.
            *t = Self::new(None);

            // b. Return NaN.
            return Ok(t.as_value());
        };

        // 6. If 0 ≤ yi ≤ 99, let yyyy be 1900𝔽 + 𝔽(yi).
        // 7. Else, let yyyy be y.
        if (0..=99).contains(&year) {
            year += 1900;
        }

        // 8. Let d be MakeDay(yyyy, MonthFromTime(t), DateFromTime(t)).
        // 9. Let date be UTC(MakeDate(d, TimeWithinDay(t))).
        let datetime = replace_params::<true>(
            datetime.timestamp_millis(),
            DateParameters {
                year: Some(IntegerOrNan::Integer(year)),
                ..Default::default()
            },
            context.host_hooks(),
        );

        // 10. Set the [[DateValue]] internal slot of this Date object to TimeClip(date).
        *t = Self::new(datetime);

        // 11. Return the value of the [[DateValue]] internal slot of this Date object.
        Ok(t.as_value())
    }

    /// [`Date.prototype.setTime()`][spec].
    ///
    /// The `setTime()` method sets the Date object to the time represented by a number of milliseconds
    /// since January 1, 1970, 00:00:00 UTC.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.settime
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setTime
    pub(crate) fn set_time(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Perform ? thisTimeValue(this value).
        let mut t = get_mut_date!(this);

        // 2. Let t be ? ToNumber(time).
        // 3. Let v be TimeClip(t).
        let timestamp = args
            .get_or_undefined(0)
            .to_integer_or_nan(context)?
            .as_integer()
            .and_then(time_clip);

        // 4. Set the [[DateValue]] internal slot of this Date object to v.
        *t = Self::new(timestamp);

        // 5. Return v.
        Ok(t.as_value())
    }

    /// [`Date.prototype.toDateString()`][spec].
    ///
    /// The `toDateString()` method returns the date portion of a Date object in English.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.todatestring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toDateString
    pub(crate) fn to_date_string(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be this Date object.
        // 2. Let tv be ? thisTimeValue(O).
        let Some(tv) = this_time_value(this)?.and_then(NaiveDateTime::from_timestamp_millis) else {
            // 3. If tv is NaN, return "Invalid Date".
            return Ok(js_string!("Invalid Date").into());
        };

        // 4. Let t be LocalTime(tv).
        // 5. Return DateString(t).
        Ok(js_string!(context
            .host_hooks()
            .local_from_utc(tv)
            .format("%a %b %d %Y")
            .to_string())
        .into())
    }

    /// [`Date.prototype.toISOString()`][spec].
    ///
    /// The `toISOString()` method returns a string in simplified extended ISO format
    /// ([ISO 8601][iso8601]).
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [iso8601]: http://en.wikipedia.org/wiki/ISO_8601
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.toisostring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toISOString
    pub(crate) fn to_iso_string(
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context,
    ) -> JsResult<JsValue> {
        let t = this_time_value(this)?
            .and_then(NaiveDateTime::from_timestamp_millis)
            .ok_or_else(|| JsNativeError::range().with_message("Invalid time value"))?;
        Ok(js_string!(Utc
            .from_utc_datetime(&t)
            .format("%Y-%m-%dT%H:%M:%S.%3fZ")
            .to_string())
        .into())
    }

    /// [`Date.prototype.toJSON()`][spec].
    ///
    /// The `toJSON()` method returns a string representation of the `Date` object.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.tojson
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toJSON
    pub(crate) fn to_json(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? ToObject(this value).
        let o = this.to_object(context)?;

        // 2. Let tv be ? ToPrimitive(O, number).
        let tv = this.to_primitive(context, PreferredType::Number)?;

        // 3. If Type(tv) is Number and tv is not finite, return null.
        if let Some(number) = tv.as_number() {
            if !number.is_finite() {
                return Ok(JsValue::null());
            }
        }

        // 4. Return ? Invoke(O, "toISOString").
        let func = o.get(utf16!("toISOString"), context)?;
        func.call(this, &[], context)
    }

    /// [`Date.prototype.toLocaleDateString()`][spec].
    ///
    /// The `toLocaleDateString()` method returns the date portion of the given Date instance according
    /// to language-specific conventions.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.tolocaledatestring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toLocaleDateString
    pub(crate) fn to_locale_date_string(
        _this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsError::from_opaque(JsValue::new(js_string!(
            "Function Unimplemented"
        ))))
    }

    /// [`Date.prototype.toLocaleString()`][spec].
    ///
    /// The `toLocaleString()` method returns a string representing the specified Date object.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.tolocalestring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toLocaleString
    pub(crate) fn to_locale_string(
        _this: &JsValue,
        _: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsError::from_opaque(JsValue::new(js_string!(
            "Function Unimplemented]"
        ))))
    }

    /// [`Date.prototype.toLocaleTimeString()`][spec].
    ///
    /// The `toLocaleTimeString()` method returns the time portion of a Date object in human readable
    /// form in American English.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.tolocaletimestring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toLocaleTimeString
    pub(crate) fn to_locale_time_string(
        _this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsError::from_opaque(JsValue::new(js_string!(
            "Function Unimplemented]"
        ))))
    }

    /// [`Date.prototype.toString()`][spec].
    ///
    /// The `toString()` method returns a string representing the specified Date object.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.tostring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toString
    pub(crate) fn to_string(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let tv be ? thisTimeValue(this value).
        // 2. Return ToDateString(tv).
        let Some(tv) = this_time_value(this)?.and_then(NaiveDateTime::from_timestamp_millis) else {
            return Ok(js_string!("Invalid Date").into());
        };
        Ok(js_string!(context
            .host_hooks()
            .local_from_utc(tv)
            .format("%a %b %d %Y %H:%M:%S GMT%z")
            .to_string())
        .into())
    }

    /// [`Date.prototype.toTimeString()`][spec].
    ///
    /// The `toTimeString()` method returns the time portion of a Date object in human readable form
    /// in American English.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.totimestring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toTimeString
    pub(crate) fn to_time_string(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be this Date object.
        // 2. Let tv be ? thisTimeValue(O).
        let Some(tv) = this_time_value(this)?.and_then(NaiveDateTime::from_timestamp_millis) else {
            // 3. If tv is NaN, return "Invalid Date".
            return Ok(js_string!("Invalid Date").into());
        };

        // 4. Let t be LocalTime(tv).
        // 5. Return the string-concatenation of TimeString(t) and TimeZoneString(tv).
        Ok(js_string!(context
            .host_hooks()
            .local_from_utc(tv)
            .format("%H:%M:%S GMT%z")
            .to_string())
        .into())
    }

    /// [`Date.prototype.toUTCString()`][spec].
    ///
    /// The `toUTCString()` method returns a string representing the specified Date object.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.toutcstring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toUTCString
    pub(crate) fn to_utc_string(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be this Date object.
        let Some(t) = this_time_value(this)?.and_then(NaiveDateTime::from_timestamp_millis) else {
            // 3. If tv is NaN, return "Invalid Date".
            return Ok(js_string!("Invalid Date").into());
        };

        // 2. Let tv be ? thisTimeValue(O).
        // 4. Let weekday be the Name of the entry in Table 60 with the Number WeekDay(tv).
        // 5. Let month be the Name of the entry in Table 61 with the Number MonthFromTime(tv).
        // 6. Let day be ToZeroPaddedDecimalString(ℝ(DateFromTime(tv)), 2).
        // 7. Let yv be YearFromTime(tv).
        // 8. If yv is +0𝔽 or yv > +0𝔽, let yearSign be the empty String; otherwise, let yearSign be "-".
        // 9. Let paddedYear be ToZeroPaddedDecimalString(abs(ℝ(yv)), 4).
        // 10. Return the string-concatenation of weekday, ",", the code unit 0x0020 (SPACE), day, the
        // code unit 0x0020 (SPACE), month, the code unit 0x0020 (SPACE), yearSign, paddedYear, the code
        // unit 0x0020 (SPACE), and TimeString(tv)
        let utc_string = t.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
        Ok(JsValue::new(js_string!(utc_string)))
    }

    /// [`Date.prototype.valueOf()`][spec].
    ///
    /// The `valueOf()` method returns the primitive value of a `Date` object.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.valueof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/valueOf
    pub(crate) fn value_of(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Return ? thisTimeValue(this value).
        Ok(Self::new(this_time_value(this)?).as_value())
    }

    /// [`Date.prototype [ @@toPrimitive ] ( hint )`][spec].
    ///
    /// The <code>\[@@toPrimitive\]()</code> method converts a Date object to a primitive value.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype-@@toprimitive
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/@@toPrimitive
    pub(crate) fn to_primitive(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If Type(O) is not Object, throw a TypeError exception.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Date.prototype[@@toPrimitive] called on non object")
        })?;

        let hint = args.get_or_undefined(0);

        let try_first = match hint.as_string() {
            // 3. If hint is "string" or "default", then
            // a. Let tryFirst be string.
            Some(string) if string == utf16!("string") || string == utf16!("default") => {
                PreferredType::String
            }
            // 4. Else if hint is "number", then
            // a. Let tryFirst be number.
            Some(number) if number == utf16!("number") => PreferredType::Number,
            // 5. Else, throw a TypeError exception.
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("Date.prototype[@@toPrimitive] called with invalid hint")
                    .into())
            }
        };

        // 6. Return ? OrdinaryToPrimitive(O, tryFirst).
        o.ordinary_to_primitive(context, try_first)
    }
}
