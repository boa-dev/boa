//! An implementation of the `Temporal` proposal's Calendar builtin.

use self::iso::IsoCalendar;

use super::{
    options::{ArithmeticOverflow, TemporalUnit, TemporalUnitGroup},
    plain_date::iso::IsoDateRecord,
    PlainDate, TemporalFields,
};
use crate::{
    builtins::{
        iterable::IteratorHint,
        options::{get_option, get_options_object},
        temporal, Array, BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{internal_methods::get_prototype_from_constructor, ObjectData},
    property::{Attribute, PropertyKey},
    realm::Realm,
    string::{common::StaticJsStrings, utf16},
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_profiler::Profiler;
use rustc_hash::FxHashMap;

mod iso;
pub(crate) mod utils;

#[cfg(feature = "experimental")]
#[cfg(test)]
mod tests;

// TODO: Determine how many methods actually need the context on them while using
// `icu_calendar`.
//
// NOTE (re above's TODO): Most likely context is only going to be needed for `dateFromFields`,
// `yearMonthFromFields`, `monthDayFromFields`, `dateAdd`, and `dateUntil`.
/// A trait for implementing a Builtin Calendar's Calendar Protocol in Rust.
pub(crate) trait BuiltinCalendar {
    /// Creates a `Temporal.PlainDate` object from provided fields.
    fn date_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue>;
    /// Creates a `Temporal.PlainYearMonth` object from the provided fields.
    fn year_month_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue>;
    /// Creates a `Temporal.PlainMonthDay` object from the provided fields.
    fn month_day_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue>;
    /// Returns a `Temporal.PlainDate` based off an added date.
    fn date_add(
        &self,
        date: &PlainDate,
        duration: &temporal::DurationRecord,
        overflow: ArithmeticOverflow,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue>;
    /// Returns a `Temporal.Duration` representing the duration between two dates.
    fn date_until(
        &self,
        one: &PlainDate,
        two: &PlainDate,
        largest_unit: TemporalUnit,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue>;
    /// Returns the era for a given `temporaldatelike`.
    fn era(&self, date_like: &IsoDateRecord, context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the era year for a given `temporaldatelike`
    fn era_year(&self, date_like: &IsoDateRecord, context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the `year` for a given `temporaldatelike`
    fn year(&self, date_like: &IsoDateRecord, context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the `month` for a given `temporaldatelike`
    fn month(&self, date_like: &IsoDateRecord, context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns the `monthCode` for a given `temporaldatelike`
    fn month_code(&self, date_like: &IsoDateRecord, context: &mut Context<'_>)
        -> JsResult<JsValue>;
    /// Returns the `day` for a given `temporaldatelike`
    fn day(&self, date_like: &IsoDateRecord, context: &mut Context<'_>) -> JsResult<JsValue>;
    /// Returns a value representing the day of the week for a date.
    fn day_of_week(
        &self,
        date_like: &IsoDateRecord,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue>;
    /// Returns a value representing the day of the year for a given calendar.
    fn day_of_year(
        &self,
        date_like: &IsoDateRecord,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue>;
    /// Returns a value representing the week of the year for a given calendar.
    fn week_of_year(
        &self,
        date_like: &IsoDateRecord,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue>;
    /// Returns the year of a given week.
    fn year_of_week(
        &self,
        date_like: &IsoDateRecord,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue>;
    /// Returns the days in a week for a given calendar.
    fn days_in_week(
        &self,
        date_like: &IsoDateRecord,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue>;
    /// Returns the days in a month for a given calendar.
    fn days_in_month(
        &self,
        date_like: &IsoDateRecord,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue>;
    /// Returns the days in a year for a given calendar.
    fn days_in_year(
        &self,
        date_like: &IsoDateRecord,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue>;
    /// Returns the months in a year for a given calendar.
    fn months_in_year(
        &self,
        date_like: &IsoDateRecord,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue>;
    /// Returns whether a value is within a leap year according to the designated calendar.
    fn in_leap_year(
        &self,
        date_like: &IsoDateRecord,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue>;
    /// Resolve the `TemporalFields` for the implemented Calendar
    fn resolve_fields(&self, fields: &mut TemporalFields, r#type: &str) -> JsResult<()>;
    /// Return this calendar's a fieldName and whether it is required depending on type (date, day-month).
    fn field_descriptors(&self, r#type: &[String]) -> Vec<(String, bool)>;
    /// Return the fields to ignore for this Calendar based on provided keys.
    fn field_keys_to_ignore(&self, additional_keys: Vec<PropertyKey>) -> Vec<PropertyKey>;
    /// Debug name
    fn debug_name(&self) -> &str;
}

impl core::fmt::Debug for dyn BuiltinCalendar {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.debug_name())
    }
}

// ==== Calendar Abstractions ====

const ISO: &[u16] = utf16!("iso8601");

// NOTE: potentially move these to `Realm`, so that there can be
// host defined calendars.
// Returns a map of all available calendars.
fn available_calendars() -> FxHashMap<&'static [u16], Box<dyn BuiltinCalendar>> {
    let mut map = FxHashMap::default();
    let iso: Box<dyn BuiltinCalendar> = Box::new(IsoCalendar);
    map.insert(ISO, iso);

    map
}

// Returns if an identifier is a builtin calendar.
pub(crate) fn is_builtin_calendar(identifier: &JsString) -> bool {
    let calendars = available_calendars();
    // TODO: Potentially implement `to_ascii_lowercase`.
    calendars.contains_key(identifier.as_slice())
}

/// The `Temporal.Calendar` object.
#[derive(Debug)]
pub struct Calendar {
    identifier: JsString,
}

impl BuiltInObject for Calendar {
    const NAME: JsString = StaticJsStrings::CALENDAR;
}

impl IntrinsicObject for Calendar {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        let get_id = BuiltInBuilder::callable(realm, Self::get_id)
            .name(js_string!("get Id"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::CONFIGURABLE,
            )
            .accessor(utf16!("id"), Some(get_id), None, Attribute::default())
            .method(Self::date_from_fields, js_string!("dateFromFields"), 2)
            .method(
                Self::year_month_from_fields,
                js_string!("yearMonthFromFields"),
                2,
            )
            .method(
                Self::month_day_from_fields,
                js_string!("monthDayFromFields"),
                2,
            )
            .method(Self::date_add, js_string!("dateAdd"), 3)
            .method(Self::date_until, js_string!("dateUntil"), 3)
            .method(Self::era, js_string!("era"), 1)
            .method(Self::era_year, js_string!("eraYear"), 1)
            .method(Self::year, js_string!("year"), 1)
            .method(Self::month, js_string!("month"), 1)
            .method(Self::month_code, js_string!("monthCode"), 1)
            .method(Self::day, js_string!("day"), 1)
            .method(Self::day_of_week, js_string!("dayOfWeek"), 1)
            .method(Self::day_of_year, js_string!("dayOfYear"), 1)
            .method(Self::week_of_year, js_string!("weekOfYear"), 1)
            .method(Self::year_of_week, js_string!("yearOfWeek"), 1)
            .method(Self::days_in_week, js_string!("daysInWeek"), 1)
            .method(Self::days_in_month, js_string!("daysInMonth"), 1)
            .method(Self::days_in_year, js_string!("daysInYear"), 1)
            .method(Self::months_in_year, js_string!("monthsInYear"), 1)
            .method(Self::in_leap_year, js_string!("inLeapYear"), 1)
            .method(Self::fields, js_string!("fields"), 1)
            .method(Self::merge_fields, js_string!("mergeFields"), 2)
            .method(Self::get_id, js_string!("toString"), 0)
            .method(Self::get_id, js_string!("toJSON"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for Calendar {
    const LENGTH: usize = 1;

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
        let JsValue::String(id) = identifier else {
            return Err(JsNativeError::typ()
                .with_message("Calendar id must be a string.")
                .into());
        };

        // 3. If IsBuiltinCalendar(id) is false, then
        if !is_builtin_calendar(id) {
            // a. Throw a RangeError exception.
            return Err(JsNativeError::range()
                .with_message("Calendar ID must be a valid builtin calendar.")
                .into());
        }

        // 4. Return ? CreateTemporalCalendar(id, NewTarget).
        create_temporal_calendar(id, Some(new_target.clone()), context)
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
        // 1. Let calendar be the this value.
        // 2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        // Retrieve the current CalendarProtocol.
        let available_calendars = available_calendars();
        let this_calendar = available_calendars
            .get(calendar.identifier.as_slice())
            .expect("builtin must exist");

        // 3. If Type(fields) is not Object, throw a TypeError exception.
        let fields = args.get_or_undefined(0);
        let fields_obj = fields.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("fields parameter must be an object.")
        })?;

        // 4. Set options to ? GetOptionsObject(options).
        let options = get_options_object(args.get_or_undefined(1))?;

        // 5. Let relevantFieldNames be « "day", "month", "monthCode", "year" ».
        let mut relevant_field_names = Vec::from([
            "day".to_owned(),
            "month".to_owned(),
            "monthCode".to_owned(),
            "year".to_owned(),
        ]);

        // 6. If calendar.[[Identifier]] is "iso8601", then
        let mut fields = if calendar.identifier.as_slice() == ISO {
            // a. Set fields to ? PrepareTemporalFields(fields, relevantFieldNames, « "year", "day" »).
            let mut required_fields = Vec::from(["year".to_owned(), "day".to_owned()]);
            temporal::TemporalFields::from_js_object(
                fields_obj,
                &mut relevant_field_names,
                &mut required_fields,
                None,
                false,
                None,
                context,
            )?
        // 7. Else,
        } else {
            // a. Let calendarRelevantFieldDescriptors be CalendarFieldDescriptors(calendar.[[Identifier]], date).
            let calendar_relevant_fields = this_calendar.field_descriptors(&["date".to_owned()]);
            // b. Set fields to ? PrepareTemporalFields(fields, relevantFieldNames, « », calendarRelevantFieldDescriptors).
            temporal::TemporalFields::from_js_object(
                fields_obj,
                &mut relevant_field_names,
                &mut Vec::new(),
                Some(calendar_relevant_fields),
                false,
                None,
                context,
            )?
        };

        // 8. Let overflow be ? ToTemporalOverflow(options).
        let overflow =
            get_option::<ArithmeticOverflow>(&options, utf16!("overflow"), false, context)?
                .unwrap_or(ArithmeticOverflow::Constrain);

        // NOTE: implement the below on the calenar itself
        // 9. If calendar.[[Identifier]] is "iso8601", then
        // a. Perform ? ISOResolveMonth(fields).
        // b. Let result be ? ISODateFromFields(fields, overflow).
        // 10. Else,
        // a. Perform ? CalendarResolveFields(calendar.[[Identifier]], fields, date).
        // b. Let result be ? CalendarDateToISO(calendar.[[Identifier]], fields, overflow).

        this_calendar.date_from_fields(&mut fields, overflow, context)
    }

    /// 15.8.2.2 `Temporal.Calendar.prototype.yearMonthFromFields ( fields [ , options ] )` - Supercedes 12.5.5
    fn year_month_from_fields(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_calendar = available_calendars
            .get(calendar.identifier.as_slice())
            .expect("builtin must exist");
        let fields = args.get_or_undefined(0);
        let fields_obj = fields.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("fields parameter must be an object.")
        })?;

        // 5. Set options to ? GetOptionsObject(options).
        let options = get_options_object(args.get_or_undefined(1))?;

        let mut relevant_field_names = Vec::from([
            "year".to_owned(),
            "month".to_owned(),
            "monthCode".to_owned(),
        ]);

        // 6. Set fields to ? PrepareTemporalFields(fields, « "month", "monthCode", "year" », « "year" »).
        let mut fields = if calendar.identifier.as_slice() == ISO {
            // a. Set fields to ? PrepareTemporalFields(fields, relevantFieldNames, « "year" »).
            let mut required_fields = Vec::from(["year".to_owned()]);
            temporal::TemporalFields::from_js_object(
                fields_obj,
                &mut relevant_field_names,
                &mut required_fields,
                None,
                false,
                None,
                context,
            )?
        } else {
            // a. Let calendarRelevantFieldDescriptors be CalendarFieldDescriptors(calendar.[[Identifier]], year-month).
            // b. Set fields to ? PrepareTemporalFields(fields, relevantFieldNames, « », calendarRelevantFieldDescriptors).

            let calendar_relevant_fields =
                this_calendar.field_descriptors(&["year-month".to_owned()]);
            temporal::TemporalFields::from_js_object(
                fields_obj,
                &mut relevant_field_names,
                &mut Vec::new(),
                Some(calendar_relevant_fields),
                false,
                None,
                context,
            )?
            // TODO: figure out the below. Maybe a method on fields?
            // c. Let firstDayIndex be the 1-based index of the first day of the month described by fields (i.e., 1 unless the month's first day is skipped by this calendar.)
            // d. Perform ! CreateDataPropertyOrThrow(fields, "day", 𝔽(firstDayIndex)).
        };

        // 7. Let overflow be ? ToTemporalOverflow(options).
        let overflow =
            get_option::<ArithmeticOverflow>(&options, utf16!("overflow"), false, context)?
                .unwrap_or(ArithmeticOverflow::Constrain);

        this_calendar.year_month_from_fields(&mut fields, overflow, context)
    }

    /// 15.8.2.3 `Temporal.Calendar.prototype.monthDayFromFields ( fields [ , options ] )` - Supercedes 12.5.6
    fn month_day_from_fields(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let calendar be the this value.
        // 2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_calendar = available_calendars
            .get(calendar.identifier.as_slice())
            .expect("builtin must exist");

        // 3. If Type(fields) is not Object, throw a TypeError exception.
        let fields = args.get_or_undefined(0);
        let fields_obj = fields.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("fields parameter must be an object.")
        })?;

        // 4. Set options to ? GetOptionsObject(options).
        let options = get_options_object(args.get_or_undefined(1))?;

        // 5. Let relevantFieldNames be « "day", "month", "monthCode", "year" ».
        let mut relevant_field_names = Vec::from([
            "day".to_owned(),
            "month".to_owned(),
            "monthCode".to_owned(),
            "year".to_owned(),
        ]);

        // 6. If calendar.[[Identifier]] is "iso8601", then
        let mut fields = if calendar.identifier.as_slice() == ISO {
            // a. Set fields to ? PrepareTemporalFields(fields, relevantFieldNames, « "day" »).
            let mut required_fields = Vec::from(["day".to_owned()]);
            temporal::TemporalFields::from_js_object(
                fields_obj,
                &mut relevant_field_names,
                &mut required_fields,
                None,
                false,
                None,
                context,
            )?
        // 7. Else,
        } else {
            // a. Let calendarRelevantFieldDescriptors be CalendarFieldDescriptors(calendar.[[Identifier]], month-day).
            let calendar_relevant_fields =
                this_calendar.field_descriptors(&["month-day".to_owned()]);
            // b. Set fields to ? PrepareTemporalFields(fields, relevantFieldNames, « », calendarRelevantFieldDescriptors).
            temporal::TemporalFields::from_js_object(
                fields_obj,
                &mut relevant_field_names,
                &mut Vec::new(),
                Some(calendar_relevant_fields),
                false,
                None,
                context,
            )?
        };

        // 8. Let overflow be ? ToTemporalOverflow(options).
        let overflow =
            get_option::<ArithmeticOverflow>(&options, utf16!("overflow"), false, context)?
                .unwrap_or(ArithmeticOverflow::Constrain);

        this_calendar.month_day_from_fields(&mut fields, overflow, context)
    }

    /// 15.8.2.4 `Temporal.Calendar.prototype.dateAdd ( date, duration [ , options ] )` - supercedes 12.5.7
    fn date_add(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let calendar be the this value.
        // 2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
        // 3. Assert: calendar.[[Identifier]] is "iso8601".
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_calendar = available_calendars
            .get(calendar.identifier.as_slice())
            .expect("builtin must exist");

        // 4. Set date to ? ToTemporalDate(date).
        let date_like = args.get_or_undefined(0);
        let date = temporal::plain_date::to_temporal_date(date_like, None, context)?;

        // 5. Set duration to ? ToTemporalDuration(duration).
        let duration_like = args.get_or_undefined(1);
        let mut duration = temporal::duration::to_temporal_duration(duration_like)?;

        // 6. Set options to ? GetOptionsObject(options).
        let options = args.get_or_undefined(2);
        let options_obj = get_options_object(options)?;

        // 7. Let overflow be ? ToTemporalOverflow(options).
        let overflow =
            get_option::<ArithmeticOverflow>(&options_obj, utf16!("overflow"), false, context)?
                .unwrap_or(ArithmeticOverflow::Constrain);

        // 8. Let balanceResult be ? BalanceTimeDuration(duration.[[Days]], duration.[[Hours]], duration.[[Minutes]], duration.[[Seconds]], duration.[[Milliseconds]], duration.[[Microseconds]], duration.[[Nanoseconds]], "day").
        duration.balance_time_duration(TemporalUnit::Day, None)?;

        this_calendar.date_add(&date, &duration, overflow, context)
    }

    ///15.8.2.5 `Temporal.Calendar.prototype.dateUntil ( one, two [ , options ] )` - Supercedes 12.5.8
    fn date_until(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let calendar be the this value.
        // 2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
        // 3. Assert: calendar.[[Identifier]] is "iso8601".
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_calendar = available_calendars
            .get(calendar.identifier.as_slice())
            .expect("builtin must exist");

        // 4. Set one to ? ToTemporalDate(one).
        let one = temporal::plain_date::to_temporal_date(args.get_or_undefined(0), None, context)?;
        // 5. Set two to ? ToTemporalDate(two).
        let two = temporal::plain_date::to_temporal_date(args.get_or_undefined(1), None, context)?;

        // 6. Set options to ? GetOptionsObject(options).
        let options = get_options_object(args.get_or_undefined(2))?;

        // 7. Let largestUnit be ? GetTemporalUnit(options, "largestUnit", date, "auto").
        // 8. If largestUnit is "auto", set largestUnit to "day".
        let largest_unit = super::options::get_temporal_unit(
            &options,
            utf16!("largestUnit"),
            TemporalUnitGroup::Date,
            Some(TemporalUnit::Day),
            None,
            context,
        )?;

        this_calendar.date_until(&one, &two, largest_unit, context)
    }

    /// 15.8.2.6 `Temporal.Calendar.prototype.era ( temporalDateLike )`
    fn era(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_calendar = available_calendars
            .get(calendar.identifier.as_slice())
            .expect("builtin must exist");

        let date_like = args.get_or_undefined(0);

        let date_info = match date_like {
            JsValue::Object(o) if o.is_plain_date_time() => {
                let obj = o.borrow();
                let date_time = obj.as_plain_date_time().expect("obj must be a DateTime.");

                date_time.inner.iso_date()
            }
            JsValue::Object(o) if o.is_plain_date() => {
                let obj = o.borrow();
                let date = obj.as_plain_date().expect("Must be a Date");

                date.inner
            }
            JsValue::Object(o) if o.is_plain_year_month() => {
                let obj = o.borrow();
                let ym = obj.as_plain_year_month().expect("must be a YearMonth.");

                ym.inner
            }
            _ => {
                let date = temporal::plain_date::to_temporal_date(date_like, None, context)?;
                date.inner
            }
        };

        this_calendar.era(&date_info, context)
    }

    /// 15.8.2.7 `Temporal.Calendar.prototype.eraYear ( temporalDateLike )`
    fn era_year(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_calendar = available_calendars
            .get(calendar.identifier.as_slice())
            .expect("builtin must exist");

        let date_like = args.get_or_undefined(0);

        let date_info = match date_like {
            JsValue::Object(o) if o.is_plain_date_time() => {
                let obj = o.borrow();
                let date_time = obj.as_plain_date_time().expect("obj must be a DateTime.");

                date_time.inner.iso_date()
            }
            JsValue::Object(o) if o.is_plain_date() => {
                let obj = o.borrow();
                let date = obj.as_plain_date().expect("Must be a Date");

                date.inner
            }
            JsValue::Object(o) if o.is_plain_year_month() => {
                let obj = o.borrow();
                let ym = obj.as_plain_year_month().expect("must be a YearMonth.");

                ym.inner
            }
            _ => {
                let date = temporal::plain_date::to_temporal_date(date_like, None, context)?;
                date.inner
            }
        };

        this_calendar.era_year(&date_info, context)
    }

    /// 15.8.2.8 `Temporal.Calendar.prototype.year ( temporalDateLike )`
    fn year(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_calendar = available_calendars
            .get(calendar.identifier.as_slice())
            .expect("builtin must exist");

        let date_like = args.get_or_undefined(0);

        let date_record = match date_like {
            JsValue::Object(o) if o.is_plain_date_time() => {
                let obj = o.borrow();
                let date_time = obj.as_plain_date_time().expect("obj must be a DateTime.");

                date_time.inner.iso_date()
            }
            JsValue::Object(o) if o.is_plain_date() => {
                let obj = o.borrow();
                let date = obj.as_plain_date().expect("Must be a Date");

                date.inner
            }
            JsValue::Object(o) if o.is_plain_year_month() => {
                let obj = o.borrow();
                let ym = obj.as_plain_year_month().expect("must be a YearMonth.");

                ym.inner
            }
            _ => {
                let date = temporal::plain_date::to_temporal_date(date_like, None, context)?;
                date.inner
            }
        };

        this_calendar.year(&date_record, context)
    }

    /// 15.8.2.9 `Temporal.Calendar.prototype.month ( temporalDateLike )`
    fn month(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_calendar = available_calendars
            .get(calendar.identifier.as_slice())
            .expect("builtin must exist");

        let date_like = args.get_or_undefined(0);

        // 3. If Type(temporalDateLike) is Object and temporalDateLike has an [[InitializedTemporalMonthDay]] internal slot, then
        // 3.a. Throw a TypeError exception.
        // 4. If Type(temporalDateLike) is not Object or temporalDateLike does not have an [[InitializedTemporalDate]], [[InitializedTemporalDateTime]], or [[InitializedTemporalYearMonth]] internal slot, then
        // 4.a. Set temporalDateLike to ? ToTemporalDate(temporalDateLike).
        let date_record = match date_like {
            JsValue::Object(o) if o.is_plain_date_time() => {
                let obj = o.borrow();
                let date_time = obj.as_plain_date_time().expect("obj must be a DateTime.");

                date_time.inner.iso_date()
            }
            JsValue::Object(o) if o.is_plain_date() => {
                let obj = o.borrow();
                let date = obj.as_plain_date().expect("Must be a Date");

                date.inner
            }
            JsValue::Object(o) if o.is_plain_year_month() => {
                let obj = o.borrow();
                let ym = obj.as_plain_year_month().expect("must be a YearMonth.");

                ym.inner
            }
            JsValue::Object(o) if o.is_plain_month_day() => {
                return Err(JsNativeError::typ()
                    .with_message("month cannot be called with PlainMonthDay object.")
                    .into())
            }
            _ => {
                let date = temporal::plain_date::to_temporal_date(date_like, None, context)?;
                date.inner
            }
        };

        this_calendar.month(&date_record, context)
    }

    /// 15.8.2.10 `Temporal.Calendar.prototype.monthCode ( temporalDateLike )`
    fn month_code(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_calendar = available_calendars
            .get(calendar.identifier.as_slice())
            .expect("builtin must exist");

        let date_like = args.get_or_undefined(0);

        let date_record = match date_like {
            JsValue::Object(o) if o.is_plain_date_time() => {
                let obj = o.borrow();
                let date_time = obj.as_plain_date_time().expect("obj must be a DateTime.");

                date_time.inner.iso_date()
            }
            JsValue::Object(o) if o.is_plain_date() => {
                let obj = o.borrow();
                let date = obj.as_plain_date().expect("Must be a Date");

                date.inner
            }
            JsValue::Object(o) if o.is_plain_year_month() => {
                let obj = o.borrow();
                let ym = obj.as_plain_year_month().expect("must be a YearMonth.");

                ym.inner
            }
            JsValue::Object(o) if o.is_plain_month_day() => {
                let obj = o.borrow();
                let md = obj.as_plain_month_day().expect("must be a MonthDay.");

                md.inner
            }
            _ => {
                let date = temporal::plain_date::to_temporal_date(date_like, None, context)?;
                date.inner
            }
        };

        this_calendar.month_code(&date_record, context)
    }

    /// 15.8.2.11 `Temporal.Calendar.prototype.day ( temporalDateLike )`
    fn day(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_calendar = available_calendars
            .get(calendar.identifier.as_slice())
            .expect("builtin must exist");

        let date_like = args.get_or_undefined(0);

        let date_record = match date_like {
            JsValue::Object(o) if o.is_plain_date_time() => {
                let obj = o.borrow();
                let date_time = obj.as_plain_date_time().expect("obj must be a DateTime.");

                date_time.inner.iso_date()
            }
            JsValue::Object(o) if o.is_plain_date() => {
                let obj = o.borrow();
                let date = obj.as_plain_date().expect("Must be a Date");

                date.inner
            }
            JsValue::Object(o) if o.is_plain_month_day() => {
                let obj = o.borrow();
                let md = obj.as_plain_month_day().expect("must be a MonthDay.");

                md.inner
            }
            _ => {
                let date = temporal::plain_date::to_temporal_date(date_like, None, context)?;
                date.inner
            }
        };

        this_calendar.day(&date_record, context)
    }

    /// 15.8.2.12 `Temporal.Calendar.prototype.dayOfWeek ( dateOrDateTime )`
    fn day_of_week(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let calendar be the this value.
        // 2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_calendar = available_calendars
            .get(calendar.identifier.as_slice())
            .expect("builtin must exist");

        // 3. Let temporalDate be ? ToTemporalDate(temporalDateLike).
        let date = temporal::plain_date::to_temporal_date(args.get_or_undefined(0), None, context)?;

        this_calendar.day_of_week(&date.inner, context)
    }

    /// 15.8.2.13 `Temporal.Calendar.prototype.dayOfYear ( temporalDateLike )`
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

        let this_calendar = available_calendars
            .get(calendar.identifier.as_slice())
            .expect("builtin must exist");

        // 3. Let temporalDate be ? ToTemporalDate(temporalDateLike).
        let date = temporal::plain_date::to_temporal_date(args.get_or_undefined(0), None, context)?;

        this_calendar.day_of_year(&date.inner, context)
    }

    /// 15.8.2.14 `Temporal.Calendar.prototype.weekOfYear ( temporalDateLike )`
    fn week_of_year(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_calendar = available_calendars
            .get(calendar.identifier.as_slice())
            .expect("builtin must exist");

        // 3. Let temporalDate be ? ToTemporalDate(temporalDateLike).
        let date = temporal::plain_date::to_temporal_date(args.get_or_undefined(0), None, context)?;

        this_calendar.week_of_year(&date.inner, context)
    }

    /// 15.8.2.15 `Temporal.Calendar.prototype.yearOfWeek ( temporalDateLike )`
    fn year_of_week(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_calendar = available_calendars
            .get(calendar.identifier.as_slice())
            .expect("builtin must exist");

        // 3. Let temporalDate be ? ToTemporalDate(temporalDateLike).
        let date = temporal::plain_date::to_temporal_date(args.get_or_undefined(0), None, context)?;

        this_calendar.year_of_week(&date.inner, context)
    }

    /// 15.8.2.16 `Temporal.Calendar.prototype.daysInWeek ( temporalDateLike )`
    fn days_in_week(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_calendar = available_calendars
            .get(calendar.identifier.as_slice())
            .expect("builtin must exist");

        // 3. Let temporalDate be ? ToTemporalDate(temporalDateLike).
        let date = temporal::plain_date::to_temporal_date(args.get_or_undefined(0), None, context)?;

        this_calendar.days_in_week(&date.inner, context)
    }

    /// 15.8.2.17 `Temporal.Calendar.prototype.daysInMonth ( temporalDateLike )`
    fn days_in_month(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_calendar = available_calendars
            .get(calendar.identifier.as_slice())
            .expect("builtin must exist");

        let date_like = args.get_or_undefined(0);

        let date_record = match date_like {
            JsValue::Object(o) if o.is_plain_date_time() => {
                let obj = o.borrow();
                let date_time = obj.as_plain_date_time().expect("obj must be a DateTime.");

                date_time.inner.iso_date()
            }
            JsValue::Object(o) if o.is_plain_date() => {
                let obj = o.borrow();
                let date = obj.as_plain_date().expect("Must be a Date");

                date.inner
            }
            JsValue::Object(o) if o.is_plain_year_month() => {
                let obj = o.borrow();
                let ym = obj.as_plain_year_month().expect("must be a YearMonth.");

                ym.inner
            }
            _ => {
                let date = temporal::plain_date::to_temporal_date(date_like, None, context)?;
                date.inner
            }
        };

        this_calendar.days_in_month(&date_record, context)
    }

    /// 15.8.2.18 `Temporal.Calendar.prototype.daysInYear ( temporalDateLike )`
    fn days_in_year(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_calendar = available_calendars
            .get(calendar.identifier.as_slice())
            .expect("builtin must exist");

        let date_like = args.get_or_undefined(0);

        let date_record = match date_like {
            JsValue::Object(o) if o.is_plain_date_time() => {
                let obj = o.borrow();
                let date_time = obj.as_plain_date_time().expect("obj must be a DateTime.");

                date_time.inner.iso_date()
            }
            JsValue::Object(o) if o.is_plain_date() => {
                let obj = o.borrow();
                let date = obj.as_plain_date().expect("Must be a Date");

                date.inner
            }
            JsValue::Object(o) if o.is_plain_year_month() => {
                let obj = o.borrow();
                let ym = obj.as_plain_year_month().expect("must be a YearMonth.");

                ym.inner
            }
            _ => {
                let date = temporal::plain_date::to_temporal_date(date_like, None, context)?;
                date.inner
            }
        };

        this_calendar.days_in_year(&date_record, context)
    }

    /// 15.8.2.19 `Temporal.Calendar.prototype.monthsInYear ( temporalDateLike )`
    fn months_in_year(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_calendar = available_calendars
            .get(calendar.identifier.as_slice())
            .expect("builtin must exist");

        let date_like = args.get_or_undefined(0);

        let date_record = match date_like {
            JsValue::Object(o) if o.is_plain_date_time() => {
                let obj = o.borrow();
                let date_time = obj.as_plain_date_time().expect("obj must be a DateTime.");

                date_time.inner.iso_date()
            }
            JsValue::Object(o) if o.is_plain_date() => {
                let obj = o.borrow();
                let date = obj.as_plain_date().expect("Must be a Date");

                date.inner
            }
            JsValue::Object(o) if o.is_plain_year_month() => {
                let obj = o.borrow();
                let ym = obj.as_plain_year_month().expect("must be a YearMonth.");

                ym.inner
            }
            _ => {
                let date = temporal::plain_date::to_temporal_date(date_like, None, context)?;
                date.inner
            }
        };

        this_calendar.months_in_year(&date_record, context)
    }

    /// 15.8.2.20 `Temporal.Calendar.prototype.inLeapYear ( temporalDateLike )`
    fn in_leap_year(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_calendar = available_calendars
            .get(calendar.identifier.as_slice())
            .expect("builtin must exist");

        let date_like = args.get_or_undefined(0);

        let date_record = match date_like {
            JsValue::Object(o) if o.is_plain_date_time() => {
                let obj = o.borrow();
                let date_time = obj.as_plain_date_time().expect("obj must be a DateTime.");

                date_time.inner.iso_date()
            }
            JsValue::Object(o) if o.is_plain_date() => {
                let obj = o.borrow();
                let date = obj.as_plain_date().expect("Must be a Date");

                date.inner
            }
            JsValue::Object(o) if o.is_plain_year_month() => {
                let obj = o.borrow();
                let ym = obj.as_plain_year_month().expect("must be a YearMonth.");

                ym.inner
            }
            _ => {
                let date = temporal::plain_date::to_temporal_date(date_like, None, context)?;
                date.inner
            }
        };

        this_calendar.in_leap_year(&date_record, context)
    }

    /// 15.8.2.21 `Temporal.Calendar.prototype.fields ( fields )`
    fn fields(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let calendar be the this value.
        // 2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_calendar = available_calendars
            .get(calendar.identifier.as_slice())
            .expect("builtin must exist");

        // 3. Let iteratorRecord be ? GetIterator(fields, sync).
        let mut iterator_record =
            args.get_or_undefined(0)
                .get_iterator(context, Some(IteratorHint::Sync), None)?;

        // 4. Let fieldNames be a new empty List.
        let mut fields_names = Vec::new();

        // 5. Let next be true.
        // 6. Repeat, while next is not false,
        while iterator_record.step(context)? {
            // a. Set next to ? IteratorStep(iteratorRecord).
            // b. If next is not false, then
            // i. Let nextValue be ? IteratorValue(next).
            let next_value = iterator_record.value(context)?;

            // ii. If Type(nextValue) is not String, then
            if let JsValue::String(value) = next_value {
                // iii. If fieldNames contains nextValue, then
                // 1. Let completion be ThrowCompletion(a newly created RangeError object).
                // 2. Return ? IteratorClose(iteratorRecord, completion).
                // iv. If nextValue is not one of "year", "month", "monthCode", or "day", then
                // 1. Let completion be ThrowCompletion(a newly created RangeError object).
                // 2. Return ? IteratorClose(iteratorRecord, completion).
                // v. Append nextValue to the end of the List fieldNames.
                let this_name = value.to_std_string_escaped();
                match this_name.as_str() {
                    "year" | "month" | "monthCode" | "day"
                        if !fields_names.contains(&this_name) =>
                    {
                        fields_names.push(this_name);
                    }
                    _ => {
                        let completion = Err(JsNativeError::range()
                            .with_message("Invalid field name string.")
                            .into());
                        return iterator_record.close(completion, context);
                    }
                }
            } else {
                // 1. Let completion be ThrowCompletion(a newly created TypeError object).
                let completion = Err(JsNativeError::typ()
                    .with_message("field must be of type string")
                    .into());
                // 2. Return ? IteratorClose(iteratorRecord, completion).
                return iterator_record.close(completion, context);
            }
        }

        // 7. Let result be fieldNames.
        // 8. If calendar.[[Identifier]] is not "iso8601", then
        if calendar.identifier.as_slice() != ISO {
            // a. NOTE: Every built-in calendar preserves all input field names in output.
            // b. Let extraFieldDescriptors be CalendarFieldDescriptors(calendar.[[Identifier]], fieldNames).
            let extended_fields = this_calendar.field_descriptors(&fields_names);
            // c. For each Calendar Field Descriptor Record desc of extraFieldDescriptors, do
            for descriptor in extended_fields {
                // i. Append desc.[[Property]] to result.
                fields_names.push(descriptor.0);
            }
        }

        // 9. Return CreateArrayFromList(result).
        Ok(Array::create_array_from_list(
            fields_names
                .iter()
                .map(|s| JsString::from(s.clone()).into()),
            context,
        )
        .into())
    }

    /// 15.8.2.22 `Temporal.Calendar.prototype.mergeFields ( fields, additionalFields )`
    fn merge_fields(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let calendar be the this value.
        // 2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Calendar must be an object.")
        })?;
        let calendar = o.as_calendar().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("the this value of Calendar must be a Calendar object.")
        })?;

        let available_calendars = available_calendars();

        let this_calendar = available_calendars
            .get(calendar.identifier.as_slice())
            .expect("builtin must exist");

        let fields = args.get_or_undefined(0).to_object(context)?;
        let additional_fields = args.get_or_undefined(1).to_object(context)?;

        // 3. Let fieldsCopy be ? SnapshotOwnProperties(? ToObject(fields), null, « », « undefined »).
        let fields_copy = temporal::snapshot_own_properties(
            &fields,
            Some(Vec::new()),
            Some(Vec::from([JsValue::undefined()])),
            context,
        )?;

        // 4. Let additionalFieldsCopy be ? SnapshotOwnProperties(? ToObject(additionalFields), null, « », « undefined »).
        let additional_fields_copy = temporal::snapshot_own_properties(
            &additional_fields,
            Some(Vec::new()),
            Some(Vec::from([JsValue::undefined()])),
            context,
        )?;

        // 5. NOTE: Every property of fieldsCopy and additionalFieldsCopy is an enumerable data property with non-undefined value, but some property keys may be Symbols.
        // 6. Let additionalKeys be ! additionalFieldsCopy.[[OwnPropertyKeys]]().
        let add_keys = additional_fields_copy.__own_property_keys__(context)?;

        // 7. If calendar.[[Identifier]] is "iso8601", then
        // a. Let overriddenKeys be ISOFieldKeysToIgnore(additionalKeys).
        // 8. Else,
        // a. Let overriddenKeys be CalendarFieldKeysToIgnore(calendar, additionalKeys).
        let overridden_keys = this_calendar.field_keys_to_ignore(add_keys);

        // 9. Let merged be OrdinaryObjectCreate(null).
        let merged = JsObject::with_null_proto();

        // 10. NOTE: The following steps ensure that property iteration order of merged
        // matches that of fields as modified by omitting overridden properties and
        // appending non-overlapping properties from additionalFields in iteration order.
        // 11. Let fieldsKeys be ! fieldsCopy.[[OwnPropertyKeys]]().
        let field_keys = fields_copy.__own_property_keys__(context)?;
        // 12. For each element key of fieldsKeys, do
        for key in field_keys {
            // a. Let propValue be undefined.
            // b. If overriddenKeys contains key, then
            let prop_value = if overridden_keys.contains(&key) {
                // i. Set propValue to ! Get(additionalFieldsCopy, key).
                additional_fields_copy.get(key.clone(), context)?
            // c. Else,
            } else {
                // i. Set propValue to ! Get(fieldsCopy, key).
                fields_copy.get(key.clone(), context)?
            };

            // d. If propValue is not undefined, perform ! CreateDataPropertyOrThrow(merged, key, propValue).
            if !prop_value.is_undefined() {
                merged.create_data_property_or_throw(key, prop_value, context)?;
            }
        }

        // 13. Perform ! CopyDataProperties(merged, additionalFieldsCopy, « »).
        temporal::copy_data_properties(
            &merged,
            &additional_fields_copy.into(),
            &Vec::new(),
            None,
            context,
        )?;

        // 14. Return merged.
        Ok(merged.into())
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
    assert!(is_builtin_calendar(identifier));

    let calendar = Calendar {
        identifier: identifier.clone(),
    };
    // 2. If newTarget is not provided, set newTarget to %Temporal.Calendar%.
    let new_target = new_target.unwrap_or_else(|| {
        context
            .realm()
            .intrinsics()
            .constructors()
            .calendar()
            .constructor()
            .into()
    });

    // 3. Let object be ? OrdinaryCreateFromConstructor(newTarget, "%Temporal.Calendar.prototype%", « [[InitializedTemporalCalendar]], [[Identifier]] »).
    let proto =
        get_prototype_from_constructor(&new_target, StandardConstructors::calendar, context)?;

    let obj = JsObject::from_proto_and_data(proto, ObjectData::calendar(calendar));

    // 4. Set object.[[Identifier]] to the ASCII-lowercase of identifier.
    // 5. Return object.
    Ok(obj.into())
}

/// 12.2.21 `GetTemporalCalendarSlotValueWithISODefault ( item )`
#[allow(unused)]
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
    let calendar_like = item.get(utf16!("calendar"), context)?;

    // 3. Return ? ToTemporalCalendarSlotValue(calendarLike, "iso8601").
    to_temporal_calendar_slot_value(&calendar_like, Some(ISO.into()))
}

#[allow(unused)]
fn to_temporal_calendar_slot_value(
    calendar_like: &JsValue,
    default: Option<JsString>,
) -> JsResult<JsValue> {
    // 1. If temporalCalendarLike is undefined and default is present, then
    if calendar_like.is_undefined() {
        if let Some(default) = default {
            // a. Assert: IsBuiltinCalendar(default) is true.
            if is_builtin_calendar(&default) {
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
                return Ok(calendar);
            }
        } else if calendar_like.is_plain_date_time() {
            let obj = calendar_like.borrow();
            let date_time = obj.as_plain_date_time();
            if let Some(dt) = date_time {
                let calendar = dt.calendar.clone();
                return Ok(calendar);
            }
        } else if calendar_like.is_plain_year_month() {
            let obj = calendar_like.borrow();
            let year_month = obj.as_plain_year_month();
            if let Some(ym) = year_month {
                let calendar = ym.calendar.clone();
                return Ok(calendar);
            }
        } else if calendar_like.is_plain_month_day() {
            let obj = calendar_like.borrow();
            let month_day = obj.as_plain_month_day();
            if let Some(md) = month_day {
                let calendar = md.calendar.clone();
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
    Ok(js_string!(ISO).into())
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
#[allow(unused)]
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
#[allow(unused)]
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
#[allow(unused)]
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
#[allow(unused)]
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
#[allow(unused)]
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
            .into());
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
#[allow(unused)]
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
            .into());
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
#[allow(unused)]
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
#[allow(unused)]
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
            .into());
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
#[allow(unused)]
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
            .into());
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
#[allow(unused)]
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
            .into());
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
#[allow(unused)]
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
            .into());
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
#[allow(unused)]
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
            .into());
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
#[allow(unused)]
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
            .into());
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
#[allow(unused)]
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
            .into());
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
#[allow(unused)]
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
            .into());
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
#[allow(unused)]
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
            .into());
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
#[allow(unused)]
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
#[allow(unused)]
pub(crate) fn calendar_date_from_fields(
    _calendar: &JsValue,
    _fields: &JsObject,
    options: Option<&JsValue>,
    _date_from_fields: Option<&JsObject>,
) -> JsResult<PlainDate> {
    let _options = match options {
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
