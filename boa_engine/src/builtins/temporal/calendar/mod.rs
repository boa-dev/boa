#![allow(dead_code, unused_variables)]
use std::iter;

use self::iso::IsoCalendar;

use super::{PlainDate, TemporalFields};
use crate::{
    builtins::{
        iterable::IteratorHint, Array, BuiltInBuilder, BuiltInConstructor, BuiltInObject,
        IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{internal_methods::get_prototype_from_constructor, ObjectData},
    property::{Attribute, PropertyKey},
    realm::Realm,
    string::utf16,
    Context, JsArgs, JsBigInt, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_profiler::Profiler;
use rustc_hash::FxHashMap;

mod iso;
pub(crate) mod utils;

/// A trait for implementing a Builtin Calendar's Calendar Protocol in Rust.
pub trait BuiltinCalendar {
    /// Creates a `Temporal.PlainDate` object from provided fields.
    fn date_from_fields(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Creates a `Temporal.PlainYearMonth` object from the provided fields.
    fn year_month_from_fields(
        &self,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue>;
    /// Creates a `Temporal.PlainMonthDay` object from the provided fields.
    fn month_day_from_fields(
        &self,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue>;
    /// Returns a `Temporal.PlainDate` based off an added date.
    fn date_add(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns a `Temporal.Duration` representing the duration between two dates.
    fn date_until(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the era for a given `temporaldatelike`.
    fn era(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the era year for a given `temporaldatelike`
    fn era_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the `year` for a given `temporaldatelike`
    fn year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the `month` for a given `temporaldatelike`
    fn month(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the `monthCode` for a given `temporaldatelike`
    fn month_code(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the `day` for a given `temporaldatelike`
    fn day(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns a value representing the day of the week for a date.
    fn day_of_week(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns a value representing the day of the year for a given calendar.
    fn day_of_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns a value representing the week of the year for a given calendar.
    fn week_of_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the year of a given week.
    fn year_of_week(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the days in a week for a given calendar.
    fn days_in_week(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the days in a month for a given calendar.
    fn days_in_month(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the days in a year for a given calendar.
    fn days_in_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the months in a year for a given calendar.
    fn months_in_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns whether a value is within a leap year according to the designated calendar.
    fn in_leap_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the fields of the implemented calendar
    fn fields(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Merges provided fields.
    fn merge_fields(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue>;
}

impl core::fmt::Debug for dyn BuiltinCalendar {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Builtin Calendar Protocol")
    }
}

// ==== Calendar Abstractions ====

// NOTE: potentially move these to `Realm`, so that there can be
// host defined calendars.
// Returns a map of all available calendars.
fn available_calendars() -> FxHashMap<&'static str, Box<dyn BuiltinCalendar>> {
    let mut map = FxHashMap::default();
    let iso: Box<dyn BuiltinCalendar> = Box::new(IsoCalendar);
    map.insert("iso8601", iso);

    map
}

// Returns if an identifier is a builtin calendar.
pub(crate) fn is_builtin_calendar(identifier: &str) -> bool {
    let calendars = available_calendars();
    calendars.contains_key(identifier.to_ascii_lowercase().as_str())
}

/// The `Temporal.Calendar` object.
#[derive(Debug)]
pub struct Calendar {
    identifier: JsString,
}

impl BuiltInObject for Calendar {
    const NAME: &'static str = "Temporal.Calendar";
}

impl IntrinsicObject for Calendar {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let get_id = BuiltInBuilder::callable(realm, Self::get_id)
            .name("get Id")
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::CONFIGURABLE,
            )
            .accessor(utf16!("id"), Some(get_id), None, Attribute::default())
            .method(Self::date_from_fields, "dateFromFields", 2)
            .method(Self::year_month_from_fields, "yearMonthFromFields", 2)
            .method(Self::month_day_from_fields, "monthDayFromFields", 2)
            .method(Self::date_add, "dateAdd", 3)
            .method(Self::date_until, "dateUntil", 3)
            .method(Self::year, "year", 1)
            .method(Self::month, "month", 1)
            .method(Self::month_code, "monthCode", 1)
            .method(Self::day, "day", 1)
            .method(Self::day_of_week, "dayOfWeek", 1)
            .method(Self::day_of_year, "dayOfYear", 1)
            .method(Self::week_of_year, "weekOfYear", 1)
            .method(Self::year_of_week, "yearOfWeek", 1)
            .method(Self::days_in_week, "daysInWeek", 1)
            .method(Self::days_in_month, "daysInMonth", 1)
            .method(Self::days_in_year, "daysInYear", 1)
            .method(Self::months_in_year, "monthsInYear", 1)
            .method(Self::in_leap_year, "inLeapYear", 1)
            .method(Self::fields, "fields", 1)
            .method(Self::merge_fields, "mergeFields", 2)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for Calendar {
    const LENGTH: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::calendar;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, then
        if new_target.is_undefined() {
            // a. Throw a TypeError exception.
            return Err(JsNativeError::typ()
                .with_message(
                    "newTarget cannot be undefined when constructing a Temporal.Calendar object.",
                )
                .into());
        }

        let identifier = args.get_or_undefined(0);

        // 2. If id is not a String, throw a TypeError exception.
        if let Some(id) = identifier.as_string() {
            // 3. If IsBuiltinCalendar(id) is false, then
            if !is_builtin_calendar(&id.to_std_string_escaped()) {
                // a. Throw a RangeError exception.
                return Err(JsNativeError::range()
                    .with_message("Calendar ID must be a valid builtin calendar.")
                    .into());
            }

            // 4. Return ? CreateTemporalCalendar(id, NewTarget).
            create_temporal_calendar(id, Some(new_target.clone()), context)
        } else {
            Err(JsNativeError::typ()
                .with_message("Calendar id must be a string.")
                .into())
        }
    }
}

impl Calendar {
    fn get_id(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        Ok(calendar.identifier.clone().into())
    }

    /// 15.8.2.1 `Temporal.Calendar.prototype.dateFromFields ( fields [ , options ] )` - Supercedes 12.5.4
    fn date_from_fields(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_protocol = available_calendars
            .get(calendar.identifier.to_std_string_escaped().as_str())
            .expect("builtin must exist");
        this_protocol.date_from_fields(args, context)
    }

    /// 15.8.2.2 `Temporal.Calendar.prototype.yearMonthFromFields ( fields [ , options ] )` - Supercedes 12.5.5
    fn year_month_from_fields(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_protocol = available_calendars
            .get(calendar.identifier.to_std_string_escaped().as_str())
            .expect("builtin must exist");
        this_protocol.year_month_from_fields(args, context)
    }

    /// 15.8.2.3 `Temporal.Calendar.prototype.monthDayFromFields ( fields [ , options ] )` - Supercedes 12.5.6
    fn month_day_from_fields(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_protocol = available_calendars
            .get(calendar.identifier.to_std_string_escaped().as_str())
            .expect("builtin must exist");
        this_protocol.month_day_from_fields(args, context)
    }

    /// 15.8.2.4 `Temporal.Calendar.prototype.dateAdd ( date, duration [ , options ] )` - supercedes 12.5.7
    fn date_add(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_protocol = available_calendars
            .get(calendar.identifier.to_std_string_escaped().as_str())
            .expect("builtin must exist");
        this_protocol.date_add(args, context)
    }

    ///15.8.2.5 `Temporal.Calendar.prototype.dateUntil ( one, two [ , options ] )` - Supercedes 12.5.8
    fn date_until(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_protocol = available_calendars
            .get(calendar.identifier.to_std_string_escaped().as_str())
            .expect("builtin must exist");
        this_protocol.date_until(args, context)
    }

    /// 15.8.2.6 `Temporal.Calendar.prototype.era ( temporalDateLike )`
    fn era(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_protocol = available_calendars
            .get(calendar.identifier.to_std_string_escaped().as_str())
            .expect("builtin must exist");
        this_protocol.era(args, context)
    }

    /// 15.8.2.7 `Temporal.Calendar.prototype.eraYear ( temporalDateLike )`
    fn era_year(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_protocol = available_calendars
            .get(calendar.identifier.to_std_string_escaped().as_str())
            .expect("builtin must exist");
        this_protocol.era_year(args, context)
    }

    fn year(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_protocol = available_calendars
            .get(calendar.identifier.to_std_string_escaped().as_str())
            .expect("builtin must exist");
        this_protocol.year(args, context)
    }

    fn month(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_protocol = available_calendars
            .get(calendar.identifier.to_std_string_escaped().as_str())
            .expect("builtin must exist");
        this_protocol.month(args, context)
    }

    fn month_code(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_protocol = available_calendars
            .get(calendar.identifier.to_std_string_escaped().as_str())
            .expect("builtin must exist");
        this_protocol.month_code(args, context)
    }

    fn day(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_protocol = available_calendars
            .get(calendar.identifier.to_std_string_escaped().as_str())
            .expect("builtin must exist");
        this_protocol.day(args, context)
    }

    fn day_of_week(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_protocol = available_calendars
            .get(calendar.identifier.to_std_string_escaped().as_str())
            .expect("builtin must exist");
        this_protocol.day_of_week(args, context)
    }

    fn day_of_year(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_protocol = available_calendars
            .get(calendar.identifier.to_std_string_escaped().as_str())
            .expect("builtin must exist");
        this_protocol.day_of_year(args, context)
    }

    fn week_of_year(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_protocol = available_calendars
            .get(calendar.identifier.to_std_string_escaped().as_str())
            .expect("builtin must exist");
        this_protocol.week_of_year(args, context)
    }

    fn year_of_week(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_protocol = available_calendars
            .get(calendar.identifier.to_std_string_escaped().as_str())
            .expect("builtin must exist");
        this_protocol.year_of_week(args, context)
    }

    fn days_in_week(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_protocol = available_calendars
            .get(calendar.identifier.to_std_string_escaped().as_str())
            .expect("builtin must exist");
        this_protocol.days_in_week(args, context)
    }

    fn days_in_month(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_protocol = available_calendars
            .get(calendar.identifier.to_std_string_escaped().as_str())
            .expect("builtin must exist");
        this_protocol.days_in_month(args, context)
    }

    fn days_in_year(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_protocol = available_calendars
            .get(calendar.identifier.to_std_string_escaped().as_str())
            .expect("builtin must exist");
        this_protocol.days_in_year(args, context)
    }

    fn months_in_year(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_protocol = available_calendars
            .get(calendar.identifier.to_std_string_escaped().as_str())
            .expect("builtin must exist");
        this_protocol.months_in_year(args, context)
    }

    fn in_leap_year(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_protocol = available_calendars
            .get(calendar.identifier.to_std_string_escaped().as_str())
            .expect("builtin must exist");
        this_protocol.in_leap_year(args, context)
    }

    fn fields(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_protocol = available_calendars
            .get(calendar.identifier.to_std_string_escaped().as_str())
            .expect("builtin must exist");
        this_protocol.fields(args, context)
    }

    fn merge_fields(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let o = o.borrow();
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_protocol = available_calendars
            .get(calendar.identifier.to_std_string_escaped().as_str())
            .expect("builtin must exist");
        this_protocol.merge_fields(args, context)
    }
}

// -- `Calendar` Abstract Operations --

/// 12.2.1 `CreateTemporalCalendar ( identifier [ , newTarget ] )`
pub(crate) fn create_temporal_calendar(
    identifier: &JsString,
    new_target: Option<JsValue>,
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    // 1. Assert: IsBuiltinCalendar(identifier) is true.
    assert!(is_builtin_calendar(&identifier.to_std_string_escaped()));

    let calendar = Calendar {
        identifier: identifier.clone(),
    };
    // 2. If newTarget is not provided, set newTarget to %Temporal.Calendar%.
    let new_target = new_target.unwrap_or_else(||
        context
            .realm()
            .intrinsics()
            .constructors()
            .calendar()
            .constructor()
            .into(),
    );

    // 3. Let object be ? OrdinaryCreateFromConstructor(newTarget, "%Temporal.Calendar.prototype%", « [[InitializedTemporalCalendar]], [[Identifier]] »).
    let proto =
        get_prototype_from_constructor(&new_target, StandardConstructors::calendar, context)?;

    let obj = JsObject::from_proto_and_data(proto, ObjectData::calendar(calendar));

    // 4. Set object.[[Identifier]] to the ASCII-lowercase of identifier.
    // 5. Return object.
    Ok(obj.into())
}

/// 12.2.21 `GetTemporalCalendarSlotValueWithISODefault ( item )`
pub(crate) fn get_temporal_calendar_slot_value_with_default(
    item: &JsObject,
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    // 1. If item has an [[InitializedTemporalDate]], [[InitializedTemporalDateTime]], [[InitializedTemporalMonthDay]], [[InitializedTemporalYearMonth]], or [[InitializedTemporalZonedDateTime]] internal slot, then
    // a. Return item.[[Calendar]].
    if item.is_plain_date() {
        let obj = item.borrow();
        let date = obj.as_plain_date();
        if let Some(date) = date {
            let calendar = date.calendar.clone();
            drop(obj);
            return Ok(calendar);
        }
    } else if item.is_plain_date_time() {
        let obj = item.borrow();
        let date_time = obj.as_plain_date_time();
        if let Some(dt) = date_time {
            let calendar = dt.calendar.clone();
            drop(obj);
            return Ok(calendar);
        }
    } else if item.is_plain_year_month() {
        let obj = item.borrow();
        let year_month = obj.as_plain_year_month();
        if let Some(ym) = year_month {
            let calendar = ym.calendar.clone();
            drop(obj);
            return Ok(calendar);
        }
    } else if item.is_plain_month_day() {
        let obj = item.borrow();
        let month_day = obj.as_plain_month_day();
        if let Some(md) = month_day {
            let calendar = md.calendar.clone();
            drop(obj);
            return Ok(calendar);
        }
    } else if item.is_zoned_date_time() {
        return Err(JsNativeError::range()
            .with_message("Not yet implemented.")
            .into());
    }

    // 2. Let calendarLike be ? Get(item, "calendar").
    let calendar_like = item.get("calendar", context)?;

    // 3. Return ? ToTemporalCalendarSlotValue(calendarLike, "iso8601").
    to_temporal_calendar_slot_value(&calendar_like, Some(JsString::from("iso8601")))
}

fn to_temporal_calendar_slot_value(
    calendar_like: &JsValue,
    default: Option<JsString>,
) -> JsResult<JsValue> {
    // 1. If temporalCalendarLike is undefined and default is present, then
    if calendar_like.is_undefined() {
        if let Some(default) = default {
            // a. Assert: IsBuiltinCalendar(default) is true.
            if is_builtin_calendar(&default.to_std_string_escaped()) {
                // b. Return default.
                return Ok(default.into());
            }
        }
    // 2. If Type(temporalCalendarLike) is Object, then
    } else if let Some(calendar_like) = calendar_like.as_object() {
        // a. If temporalCalendarLike has an [[InitializedTemporalDate]], [[InitializedTemporalDateTime]], [[InitializedTemporalMonthDay]], [[InitializedTemporalYearMonth]], or [[InitializedTemporalZonedDateTime]] internal slot, then
        // i. Return temporalCalendarLike.[[Calendar]].
        if calendar_like.is_plain_date() {
            let obj = calendar_like.borrow();
            let date = obj.as_plain_date();
            if let Some(date) = date {
                let calendar = date.calendar.clone();
                drop(obj);
                return Ok(calendar);
            }
        } else if calendar_like.is_plain_date_time() {
            let obj = calendar_like.borrow();
            let date_time = obj.as_plain_date_time();
            if let Some(dt) = date_time {
                let calendar = dt.calendar.clone();
                drop(obj);
                return Ok(calendar);
            }
        } else if calendar_like.is_plain_year_month() {
            let obj = calendar_like.borrow();
            let year_month = obj.as_plain_year_month();
            if let Some(ym) = year_month {
                let calendar = ym.calendar.clone();
                drop(obj);
                return Ok(calendar);
            }
        } else if calendar_like.is_plain_month_day() {
            let obj = calendar_like.borrow();
            let month_day = obj.as_plain_month_day();
            if let Some(md) = month_day {
                let calendar = md.calendar.clone();
                drop(obj);
                return Ok(calendar);
            }
        } else if calendar_like.is_zoned_date_time() {
            return Err(JsNativeError::range()
                .with_message("Not yet implemented.")
                .into());
        }

        // TODO: implement ObjectImplementsTemporalCalendarProtocol
        // b. If ? ObjectImplementsTemporalCalendarProtocol(temporalCalendarLike) is false, throw a TypeError exception.
        // c. Return temporalCalendarLike.
        return Ok(calendar_like.clone().into());
    }

    // 3. If temporalCalendarLike is not a String, throw a TypeError exception.
    if !calendar_like.is_string() {
        return Err(JsNativeError::typ()
            .with_message("temporalCalendarLike is not a string.")
            .into());
    }

    // TODO: 4-6
    // 4. Let identifier be ? ParseTemporalCalendarString(temporalCalendarLike).
    // 5. If IsBuiltinCalendar(identifier) is false, throw a RangeError exception.
    // 6. Return the ASCII-lowercase of identifier.
    Ok("iso8601".into())
}

// ---------------------------- AbstractCalendar Methods ----------------------------
//
// The above refers to the functions in the Abstract Operations section of the Calendar
// spec takes either a calendar identifier or `Temporal.Calendar` and calls the a
// function that aligns with a method on `Temporal.Calendar`. These functions appear
// to be a second completely abstract builtin calendar implementation itself, so
// separating them from the other Abstract Operations seems both natural and will
// hopefully make any changes more maintainable.
//
// NOTE: Instead of creating temporal calendar it may be more efficient to retrieve
// the protocol and call the value directly in rust, something to consider.

/// A helper method to assess a identifier vs Calendar and calling a designated method.
fn call_method_on_abstract_calendar(
    calendar: &JsValue,
    method: &JsString,
    args: &[JsValue],
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    // If Calendar is a string
    let this_calendar = match calendar {
        JsValue::String(id) => create_temporal_calendar(id, None, context)?
            .as_object()
            .expect("CreateTemporalCalendar must return JsObject.")
            .clone(),
        JsValue::Object(calendar) => calendar.clone(),
        _ => unreachable!(),
    };

    let method = this_calendar.get(method.as_ref(), context)?;
    method.call(&this_calendar.into(), args, context)
}

/// 12.2.2 `CalendarFields ( calendar, fieldNames )`
///
/// Returns either a normal completion containing a List of Strings, or a throw completion.
pub(crate) fn calendar_fields(
    calendar: &JsValue,
    field_names: Vec<JsValue>,
    context: &mut Context<'_>,
) -> JsResult<Vec<JsValue>> {
    let field_names = Array::create_array_from_list(field_names, context);
    // 1. If calendar is a String, then
    // a. Set calendar to ! CreateTemporalCalendar(calendar).
    // b. Let fieldsArray be ? Call(%Temporal.Calendar.prototype.fields%, calendar, « CreateArrayFromList(fieldNames) »).
    // c. Return ! CreateListFromArrayLike(fieldsArray, « String »).
    // 2. Let fieldsArray be ? Invoke(calendar, "fields", « CreateArrayFromList(fieldNames) »).
    let fields_array = call_method_on_abstract_calendar(
        calendar,
        &JsString::from("fields"),
        &[field_names.into()],
        context,
    )?;

    // 3. Let iteratorRecord be ? GetIterator(fieldsArray, sync).
    let mut iterator_record = fields_array.get_iterator(context, Some(IteratorHint::Sync), None)?;
    // 4. Return ? IteratorToListOfType(iteratorRecord, « String »).
    super::iterator_to_list_of_types(&mut iterator_record, &[crate::value::Type::String], context)
}

/// 12.2.3 `CalendarMergeFields ( calendar, fields, additionalFields )`
///
/// Returns either a normal completion containing an Object, or a throw completion.
pub(crate) fn calendar_merge_fields(
    calendar: &JsValue,
    fields: &TemporalFields,
    additional_fields: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<JsObject> {
    // 1. If calendar is a String, then
    // a. Set calendar to ! CreateTemporalCalendar(calendar).
    // b. Return ? Call(%Temporal.Calendar.prototype.mergeFields%, calendar, « fields, additionalFields »).
    // 2. Let result be ? Invoke(calendar, "mergeFields", « fields, additionalFields »).
    let result = call_method_on_abstract_calendar(
        calendar,
        &JsString::from("mergeFields"),
        &[fields.as_object(context)?.into(), additional_fields.clone()],
        context,
    )?;

    // 3. If Type(result) is not Object, throw a TypeError exception.
    // 4. Return result.
    match result {
        JsValue::Object(o) => Ok(o),
        _ => Err(JsNativeError::typ()
            .with_message("mergeFields must return an object")
            .into()),
    }
}

/// 12.2.4 `CalendarDateAdd ( calendar, date, duration [ , options [ , dateAdd ] ] )`
///
/// Returns either a normal completion containing a `Temporal.PlainDate`, or an abrupt completion.
pub(crate) fn calendar_date_add(
    calendar: &JsValue,
    date: &JsObject,
    duration: &JsObject,
    options: Option<JsValue>,
    context: &mut Context<'_>,
) -> JsResult<JsObject> {
    // 1. If options is not present, set options to undefined.
    let options = options.unwrap_or(JsValue::undefined());

    // 2. If calendar is a String, then
    // a. Set calendar to ! CreateTemporalCalendar(calendar).
    // b. Return ? Call(%Temporal.Calendar.prototype.dateAdd%, calendar, « date, duration, options »).
    // 3. If dateAdd is not present, set dateAdd to ? GetMethod(calendar, "dateAdd").
    // 4. Let addedDate be ? Call(dateAdd, calendar, « date, duration, options »).
    let added_date = call_method_on_abstract_calendar(
        calendar,
        &JsString::from("dateAdd"),
        &[date.clone().into(), duration.clone().into(), options],
        context,
    )?;

    // 5. Perform ? RequireInternalSlot(addedDate, [[InitializedTemporalDate]]).
    // 6. Return addedDate.
    match added_date {
        JsValue::Object(o) if o.is_plain_date() => Ok(o),
        _ => Err(JsNativeError::typ()
            .with_message("dateAdd returned a value other than a Temoporal.PlainDate")
            .into()),
    }
}

/// 12.2.5 `CalendarDateUntil ( calendar, one, two, options [ , dateUntil ] )`
///
/// Returns either a normal completion containing a `Temporal.Duration`, or an abrupt completion.
pub(crate) fn calendar_date_until(
    calendar: &JsValue,
    one: &JsObject,
    two: &JsObject,
    options: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<super::duration::DurationRecord> {
    // 1. If calendar is a String, then
    // a. Set calendar to ! CreateTemporalCalendar(calendar).
    // b. Return ? Call(%Temporal.Calendar.prototype.dateUntil%, calendar, « one, two, options »).
    // 2. If dateUntil is not present, set dateUntil to ? GetMethod(calendar, "dateUntil").
    // 3. Let duration be ? Call(dateUntil, calendar, « one, two, options »).
    let duration = call_method_on_abstract_calendar(
        calendar,
        &JsString::from("dateUntil"),
        &[one.clone().into(), two.clone().into(), options.clone()],
        context,
    )?;

    // 4. Perform ? RequireInternalSlot(duration, [[InitializedTemporalDuration]]).
    // 5. Return duration.
    match duration {
        JsValue::Object(o) if o.is_duration() => {
            let obj = o.borrow();
            let dur = obj
                .as_duration()
                .expect("Value is confirmed to be a duration.");
            let record = dur.inner;
            drop(obj);
            Ok(record)
        }
        _ => Err(JsNativeError::typ()
            .with_message("Calendar dateUntil must return a Duration")
            .into()),
    }
}

/// 12.2.6 `CalendarYear ( calendar, dateLike )`
///
/// Returns either a normal completion containing an integer, or an abrupt completion.
pub(crate) fn calendar_year(
    calendar: &JsValue,
    datelike: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<f64> {
    // 1. If calendar is a String, then
    // a. Set calendar to ! CreateTemporalCalendar(calendar).
    // b. Return ? Call(%Temporal.Calendar.prototype.year%, calendar, « dateLike »).
    // 2. Let result be ? Invoke(calendar, "year", « dateLike »).
    let result = call_method_on_abstract_calendar(
        calendar,
        &JsString::from("year"),
        &[datelike.clone()],
        context,
    )?;

    // 3. If Type(result) is not Number, throw a TypeError exception.
    let Some(number) = result.as_number() else {
        return Err(JsNativeError::typ()
            .with_message("CalendarYear result must be a number.")
            .into())
    };

    // 4. If IsIntegralNumber(result) is false, throw a RangeError exception.
    if number.is_nan() || number.is_infinite() || number.fract() != 0.0 {
        return Err(JsNativeError::range()
            .with_message("CalendarYear was not integral.")
            .into());
    }

    // 5. Return ℝ(result).
    Ok(number)
}

/// 12.2.7 `CalendarMonth ( calendar, dateLike )`
pub(crate) fn calendar_month(
    calendar: &JsValue,
    datelike: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<f64> {
    // 1. If calendar is a String, then
    // a. Set calendar to ! CreateTemporalCalendar(calendar).
    // b. Return ? Call(%Temporal.Calendar.prototype.month%, calendar, « dateLike »).
    // 2. Let result be ? Invoke(calendar, "month", « dateLike »).
    let result = call_method_on_abstract_calendar(
        calendar,
        &JsString::from("month"),
        &[datelike.clone()],
        context,
    )?;

    // 3. If Type(result) is not Number, throw a TypeError exception.
    let Some(number) = result.as_number() else {
        return Err(JsNativeError::typ()
            .with_message("CalendarYear result must be a number.")
            .into())
    };

    // 4. If IsIntegralNumber(result) is false, throw a RangeError exception.
    if number.is_nan() || number.is_infinite() || number.fract() != 0.0 {
        return Err(JsNativeError::range()
            .with_message("CalendarMonth was not integral.")
            .into());
    }

    // 5. If result < 1𝔽, throw a RangeError exception.
    if number < 1.0 {
        return Err(JsNativeError::range()
            .with_message("month must be 1 or greater.")
            .into());
    }

    // 6. Return ℝ(result).
    Ok(number)
}

/// 12.2.8 `CalendarMonthCode ( calendar, dateLike )`
pub(crate) fn calendar_month_code(
    calendar: &JsValue,
    datelike: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<JsString> {
    // 1. If calendar is a String, then
    // a. Set calendar to ! CreateTemporalCalendar(calendar).
    // b. Return ? Call(%Temporal.Calendar.prototype.monthCode%, calendar, « dateLike »).
    // 2. Let result be ? Invoke(calendar, "monthCode", « dateLike »).
    let result = call_method_on_abstract_calendar(
        calendar,
        &JsString::from("monthCode"),
        &[datelike.clone()],
        context,
    )?;

    // 3. If Type(result) is not String, throw a TypeError exception.
    // 4. Return result.
    match result {
        JsValue::String(s) => Ok(s),
        _ => Err(JsNativeError::typ()
            .with_message("monthCode must be a String.")
            .into()),
    }
}

/// 12.2.9 `CalendarDay ( calendar, dateLike )`
pub(crate) fn calendar_day(
    calendar: &JsValue,
    datelike: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<f64> {
    // 1. If calendar is a String, then
    // a. Set calendar to ! CreateTemporalCalendar(calendar).
    // b. Return ? Call(%Temporal.Calendar.prototype.day%, calendar, « dateLike »).
    // 2. Let result be ? Invoke(calendar, "day", « dateLike »).
    let result = call_method_on_abstract_calendar(
        calendar,
        &JsString::from("day"),
        &[datelike.clone()],
        context,
    )?;

    // 3. If Type(result) is not Number, throw a TypeError exception.
    let Some(number) = result.as_number() else {
        return Err(JsNativeError::typ()
            .with_message("CalendarYear result must be a number.")
            .into())
    };

    // 4. If IsIntegralNumber(result) is false, throw a RangeError exception.
    if number.is_nan() || number.is_infinite() || number.fract() != 0.0 {
        return Err(JsNativeError::range()
            .with_message("CalendarDay was not integral.")
            .into());
    }

    // 5. If result < 1𝔽, throw a RangeError exception.
    if number < 1.0 {
        return Err(JsNativeError::range()
            .with_message("day must be 1 or greater.")
            .into());
    }

    // 6. Return ℝ(result).
    Ok(number)
}

/// 12.2.10 `CalendarDayOfWeek ( calendar, dateLike )`
pub(crate) fn calendar_day_of_week(
    calendar: &JsValue,
    datelike: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<f64> {
    // 1. If calendar is a String, then
    // a. Set calendar to ! CreateTemporalCalendar(calendar).
    // b. Return ? Call(%Temporal.Calendar.prototype.dayOfWeek%, calendar, « dateLike »).
    // 2. Let result be ? Invoke(calendar, "dayOfWeek", « dateLike »).
    let result = call_method_on_abstract_calendar(
        calendar,
        &JsString::from("dayOfWeek"),
        &[datelike.clone()],
        context,
    )?;

    // 3. If Type(result) is not Number, throw a TypeError exception.
    let Some(number) = result.as_number() else {
        return Err(JsNativeError::typ()
            .with_message("CalendarDayOfWeek result must be a number.")
            .into())
    };

    // 4. If IsIntegralNumber(result) is false, throw a RangeError exception.
    if number.is_nan() || number.is_infinite() || number.fract() != 0.0 {
        return Err(JsNativeError::range()
            .with_message("CalendarDayOfWeek was not integral.")
            .into());
    }

    // 5. If result < 1𝔽, throw a RangeError exception.
    if number < 1.0 {
        return Err(JsNativeError::range()
            .with_message("dayOfWeek must be 1 or greater.")
            .into());
    }

    // 6. Return ℝ(result).
    Ok(number)
}

/// 12.2.11 `CalendarDayOfYear ( calendar, dateLike )`
pub(crate) fn calendar_day_of_year(
    calendar: &JsValue,
    datelike: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<f64> {
    // 1. If calendar is a String, then
    // a. Set calendar to ! CreateTemporalCalendar(calendar).
    // b. Return ? Call(%Temporal.Calendar.prototype.dayOfYear%, calendar, « dateLike »).
    // 2. Let result be ? Invoke(calendar, "dayOfYear", « dateLike »).
    let result = call_method_on_abstract_calendar(
        calendar,
        &JsString::from("dayOfYear"),
        &[datelike.clone()],
        context,
    )?;

    // 3. If Type(result) is not Number, throw a TypeError exception.
    let Some(number) = result.as_number() else {
        return Err(JsNativeError::typ()
            .with_message("CalendarDayOfYear result must be a number.")
            .into())
    };

    // 4. If IsIntegralNumber(result) is false, throw a RangeError exception.
    if number.is_nan() || number.is_infinite() || number.fract() != 0.0 {
        return Err(JsNativeError::range()
            .with_message("CalendarDayOfYear was not integral.")
            .into());
    }

    // 5. If result < 1𝔽, throw a RangeError exception.
    if number < 1.0 {
        return Err(JsNativeError::range()
            .with_message("dayOfYear must be 1 or greater.")
            .into());
    }

    // 6. Return ℝ(result).
    Ok(number)
}

/// 12.2.12 `CalendarWeekOfYear ( calendar, dateLike )`
pub(crate) fn calendar_week_of_year(
    calendar: &JsValue,
    datelike: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<f64> {
    // 1. If calendar is a String, then
    // a. Set calendar to ! CreateTemporalCalendar(calendar).
    // b. Return ? Call(%Temporal.Calendar.prototype.weekOfYear%, calendar, « dateLike »).
    // 2. Let result be ? Invoke(calendar, "weekOfYear", « dateLike »).
    let result = call_method_on_abstract_calendar(
        calendar,
        &JsString::from("weekOfYear"),
        &[datelike.clone()],
        context,
    )?;

    // 3. If Type(result) is not Number, throw a TypeError exception.
    let Some(number) = result.as_number() else {
        return Err(JsNativeError::typ()
            .with_message("CalendarWeekOfYear result must be a number.")
            .into())
    };

    // 4. If IsIntegralNumber(result) is false, throw a RangeError exception.
    if number.is_nan() || number.is_infinite() || number.fract() != 0.0 {
        return Err(JsNativeError::range()
            .with_message("CalendarWeekOfYear was not integral.")
            .into());
    }

    // 5. If result < 1𝔽, throw a RangeError exception.
    if number < 1.0 {
        return Err(JsNativeError::range()
            .with_message("weekOfYear must be 1 or greater.")
            .into());
    }

    // 6. Return ℝ(result).
    Ok(number)
}

/// 12.2.13 `CalendarYearOfWeek ( calendar, dateLike )`
pub(crate) fn calendar_year_of_week(
    calendar: &JsValue,
    datelike: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<f64> {
    // 1. If calendar is a String, then
    // a. Set calendar to ! CreateTemporalCalendar(calendar).
    // b. Return ? Call(%Temporal.Calendar.prototype.yearOfWeek%, calendar, « dateLike »).
    // 2. Let result be ? Invoke(calendar, "yearOfWeek", « dateLike »).
    let result = call_method_on_abstract_calendar(
        calendar,
        &JsString::from("yearOfWeek"),
        &[datelike.clone()],
        context,
    )?;

    // 3. If Type(result) is not Number, throw a TypeError exception.
    let Some(number) = result.as_number() else {
        return Err(JsNativeError::typ()
            .with_message("CalendarYearOfWeek result must be a number.")
            .into())
    };

    // 4. If IsIntegralNumber(result) is false, throw a RangeError exception.
    if number.is_nan() || number.is_infinite() || number.fract() != 0.0 {
        return Err(JsNativeError::range()
            .with_message("CalendarYearOfWeek was not integral.")
            .into());
    }

    // 5. Return ℝ(result).
    Ok(number)
}

/// 12.2.14 `CalendarDaysInWeek ( calendar, dateLike )`
pub(crate) fn calendar_days_in_week(
    calendar: &JsValue,
    datelike: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<f64> {
    // 1. If calendar is a String, then
    // a. Set calendar to ! CreateTemporalCalendar(calendar).
    // b. Return ? Call(%Temporal.Calendar.prototype.daysInWeek%, calendar, « dateLike »).
    // 2. Let result be ? Invoke(calendar, "daysInWeek", « dateLike »).
    let result = call_method_on_abstract_calendar(
        calendar,
        &JsString::from("daysInWeek"),
        &[datelike.clone()],
        context,
    )?;

    // 3. If Type(result) is not Number, throw a TypeError exception.
    let Some(number) = result.as_number() else {
        return Err(JsNativeError::typ()
            .with_message("CalendarDaysInWeek result must be a number.")
            .into())
    };

    // 4. If IsIntegralNumber(result) is false, throw a RangeError exception.
    if number.is_nan() || number.is_infinite() || number.fract() != 0.0 {
        return Err(JsNativeError::range()
            .with_message("CalendarDaysInWeek was not integral.")
            .into());
    }

    // 5. If result < 1𝔽, throw a RangeError exception.
    if number < 1.0 {
        return Err(JsNativeError::range()
            .with_message("daysInWeek must be 1 or greater.")
            .into());
    }

    // 6. Return ℝ(result).
    Ok(number)
}

/// 12.2.15 `CalendarDaysInMonth ( calendar, dateLike )`
pub(crate) fn calendar_days_in_month(
    calendar: &JsValue,
    datelike: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<f64> {
    // 1. If calendar is a String, then
    // a. Set calendar to ! CreateTemporalCalendar(calendar).
    // b. Return ? Call(%Temporal.Calendar.prototype.daysInMonth%, calendar, « dateLike »).
    // 2. Let result be ? Invoke(calendar, "daysInMonth", « dateLike »).
    let result = call_method_on_abstract_calendar(
        calendar,
        &JsString::from("daysInMonth"),
        &[datelike.clone()],
        context,
    )?;

    // 3. If Type(result) is not Number, throw a TypeError exception.
    let Some(number) = result.as_number() else {
        return Err(JsNativeError::typ()
            .with_message("CalendarDaysInMonth result must be a number.")
            .into())
    };

    // 4. If IsIntegralNumber(result) is false, throw a RangeError exception.
    if number.is_nan() || number.is_infinite() || number.fract() != 0.0 {
        return Err(JsNativeError::range()
            .with_message("CalendarDaysInMonth was not integral.")
            .into());
    }

    // 5. If result < 1𝔽, throw a RangeError exception.
    if number < 1.0 {
        return Err(JsNativeError::range()
            .with_message("daysInMonth must be 1 or greater.")
            .into());
    }

    // 6. Return ℝ(result).
    Ok(number)
}

/// 12.2.16 `CalendarDaysInYear ( calendar, dateLike )`
pub(crate) fn calendar_days_in_year(
    calendar: &JsValue,
    datelike: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<f64> {
    // 1. If calendar is a String, then
    // a. Set calendar to ! CreateTemporalCalendar(calendar).
    // b. Return ? Call(%Temporal.Calendar.prototype.daysInYear%, calendar, « dateLike »).
    // 2. Let result be ? Invoke(calendar, "daysInYear", « dateLike »).
    let result = call_method_on_abstract_calendar(
        calendar,
        &JsString::from("daysInYear"),
        &[datelike.clone()],
        context,
    )?;

    // 3. If Type(result) is not Number, throw a TypeError exception.
    let Some(number) = result.as_number() else {
        return Err(JsNativeError::typ()
            .with_message("CalendarDaysInYear result must be a number.")
            .into())
    };

    // 4. If IsIntegralNumber(result) is false, throw a RangeError exception.
    if number.is_nan() || number.is_infinite() || number.fract() != 0.0 {
        return Err(JsNativeError::range()
            .with_message("CalendarDaysInYear was not integral.")
            .into());
    }

    // 5. If result < 1𝔽, throw a RangeError exception.
    if number < 1.0 {
        return Err(JsNativeError::range()
            .with_message("daysInYear must be 1 or greater.")
            .into());
    }

    // 6. Return ℝ(result).
    Ok(number)
}

/// 12.2.17 `CalendarMonthsInYear ( calendar, dateLike )`
pub(crate) fn calendar_months_in_year(
    calendar: &JsValue,
    datelike: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<f64> {
    // 1. If calendar is a String, then
    // a. Set calendar to ! CreateTemporalCalendar(calendar).
    // b. Return ? Call(%Temporal.Calendar.prototype.monthsInYear%, calendar, « dateLike »).
    // 2. Let result be ? Invoke(calendar, "monthsInYear", « dateLike »).
    let result = call_method_on_abstract_calendar(
        calendar,
        &JsString::from("monthsInYear"),
        &[datelike.clone()],
        context,
    )?;

    // 3. If Type(result) is not Number, throw a TypeError exception.
    let Some(number) = result.as_number() else {
        return Err(JsNativeError::typ()
            .with_message("CalendarMonthsInYear result must be a number.")
            .into())
    };

    // 4. If IsIntegralNumber(result) is false, throw a RangeError exception.
    if number.is_nan() || number.is_infinite() || number.fract() != 0.0 {
        return Err(JsNativeError::range()
            .with_message("CalendarMonthsInYear was not integral.")
            .into());
    }

    // 5. If result < 1𝔽, throw a RangeError exception.
    if number < 1.0 {
        return Err(JsNativeError::range()
            .with_message("monthsInYear must be 1 or greater.")
            .into());
    }

    // 6. Return ℝ(result).
    Ok(number)
}

/// 12.2.18 `CalendarInLeapYear ( calendar, dateLike )`
pub(crate) fn calendar_in_lear_year(
    calendar: &JsValue,
    datelike: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    // 1. If calendar is a String, then
    // a. Set calendar to ! CreateTemporalCalendar(calendar).
    // b. Return ? Call(%Temporal.Calendar.prototype.inLeapYear%, calendar, « dateLike »).
    // 2. Let result be ? Invoke(calendar, "inLeapYear", « dateLike »).
    let result = call_method_on_abstract_calendar(
        calendar,
        &JsString::from("inLeapYear"),
        &[datelike.clone()],
        context,
    )?;

    // 3. If Type(result) is not Boolean, throw a TypeError exception.
    // 4. Return result.
    match result {
        JsValue::Boolean(b) => Ok(b),
        _ => Err(JsNativeError::typ()
            .with_message("inLeapYear result must be a boolean.")
            .into()),
    }
}

/// 12.2.24 `CalendarDateFromFields ( calendar, fields [ , options [ , dateFromFields ] ] )`
pub(crate) fn calendar_date_from_fields(
    calendar: &JsValue,
    fields: &JsObject,
    options: Option<&JsValue>,
    date_from_fields: Option<&JsObject>,
) -> JsResult<PlainDate> {
    let options = match options {
        Some(o) => o.clone(),
        _ => JsValue::undefined(),
    };
    // 1. If options is not present, set options to undefined.
    // 2. If calendar is a String, then
    // a. Set calendar to ! CreateTemporalCalendar(calendar).
    // b. Return ? Call(%Temporal.Calendar.prototype.dateFromFields%, calendar, « fields, options »).
    // 3. If dateFromFields is not present, set dateFromFields to ? GetMethod(calendar, "dateFromFields").
    // 4. Let date be ? Call(calendar, dateFromFields, « fields, options »).
    // 5. Perform ? RequireInternalSlot(date, [[InitializedTemporalDate]]).
    // 6. Return date.

    Err(JsNativeError::range()
        .with_message("not yet implemented.")
        .into())
}
