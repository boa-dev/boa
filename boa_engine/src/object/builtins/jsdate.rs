use std::ops::Deref;

use boa_gc::{Finalize, Trace};

use crate::{
    builtins::Date,
    object::{JsObject, JsObjectType},
    Context, JsResult, JsValue,
};

/// `JsDate` is a wrapper for JavaScript `JsDate` builtin object
///
/// # Example
///
/// Create a `JsDate` object and set date to December 4 1995
///
/// ```
/// use boa_engine::{object::builtins::JsDate, Context, JsValue, JsResult};
///
/// fn main() -> JsResult<()> {
/// // JS mutable Context
/// let context = &mut Context::default();
///
/// let date = JsDate::new(context);
///
/// date.set_full_year(&[1995.into(), 11.into(), 4.into()], context)?;
///
/// assert_eq!(date.to_date_string(context)?, JsValue::from("Mon Dec 04 1995"));
///
/// Ok(())
/// }
/// ```
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsDate {
    inner: JsObject,
}

impl JsDate {
    /// Create a new `Date` object with universal time.
    #[inline]
    pub fn new(context: &mut Context) -> Self {
        let inner = Date::date_create(None, context);

        Self { inner }
    }

    /// Return a `Number` representing the milliseconds elapsed since the UNIX epoch.
    ///
    /// Same as JavaScript's `Date.now()`
    #[inline]
    pub fn now(context: &mut Context) -> JsResult<JsValue> {
        Date::now(&JsValue::Null, &[JsValue::Null], context)
    }

    // DEBUG: Uses RFC3339 internally therefore could match es6 spec of ISO8601  <========
    /// Parse a `String` representation of date.
    /// String should be ISO 8601 format.
    /// Returns the `Number` of milliseconds since UNIX epoch if `String`
    /// is valid, else return a `NaN`.
    ///
    /// Same as JavaScript's `Date.parse(value)`.
    #[inline]
    pub fn parse(value: JsValue, context: &mut Context) -> JsResult<JsValue> {
        Date::parse(&JsValue::Null, &[value], context)
    }

    /// Takes a [year, month, day, hour, minute, second, millisecond]
    /// Return a `Number` representing the milliseconds elapsed since the UNIX epoch.
    ///
    /// Same as JavaScript's `Date.UTC()`
    #[inline]
    pub fn utc(values: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Date::utc(&JsValue::Null, values, context)
    }

    /// Returns the day of the month(1-31) for the specified date
    /// according to local time.
    ///
    /// Same as JavaScript's `Date.prototype.getDate()`.
    #[inline]
    pub fn get_date(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::get_date(&self.inner.clone().into(), &[JsValue::null()], context)
    }

    /// Returns the day of the week (0–6) for the specified date
    /// according to local time.
    ///
    /// Same as JavaScript's `Date.prototype.getDay()`.
    #[inline]
    pub fn get_day(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::get_day(&self.inner.clone().into(), &[JsValue::null()], context)
    }

    /// Returns the year (4 digits for 4-digit years) of the specified date
    /// according to local time.
    ///
    /// Same as JavaScript's `Date.prototype.getFullYear()`.
    #[inline]
    pub fn get_full_year(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::get_full_year(&self.inner.clone().into(), &[JsValue::null()], context)
    }

    /// Returns the hour (0–23) in the specified date according to local time.
    ///
    /// Same as JavaScript's `Date.prototype.getHours()`.
    #[inline]
    pub fn get_hours(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::get_hours(&self.inner.clone().into(), &[JsValue::null()], context)
    }

    /// Returns the milliseconds (0–999) in the specified date according
    /// to local time.
    ///
    /// Same as JavaScript's `Date.prototype.getMilliseconds()`.
    #[inline]
    pub fn get_milliseconds(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::get_milliseconds(&self.inner.clone().into(), &[JsValue::null()], context)
    }

    /// Returns the minutes (0–59) in the specified date according to local time.
    ///
    /// Same as JavaScript's `Date.prototype.getMinutes()`.
    #[inline]
    pub fn get_minutes(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::get_minutes(&self.inner.clone().into(), &[JsValue::null()], context)
    }

    /// Returns the month (0–11) in the specified date according to local time.
    ///
    /// Same as JavaScript's `Date.prototype.getMonth()`.
    #[inline]
    pub fn get_month(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::get_month(&self.inner.clone().into(), &[JsValue::null()], context)
    }

    /// Returns the seconds (0–59) in the specified date according to local time.
    ///
    /// Same as JavaScript's `Date.prototype.getSeconds()`.
    #[inline]
    pub fn get_seconds(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::get_seconds(&self.inner.clone().into(), &[JsValue::null()], context)
    }

    /// Returns the numeric value of the specified date as the number
    /// of milliseconds since UNIX epoch.
    /// Negative values are returned for prior times.
    ///
    /// Same as JavaScript's `Date.prototype.getTime()`.
    #[inline]
    pub fn get_time(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::get_time(&self.inner.clone().into(), &[JsValue::null()], context)
    }

    /// Returns the time-zone offset in minutes for the current locale.
    ///
    /// Same as JavaScript's `Date.prototype.getTimezoneOffset()`.
    #[inline]
    pub fn get_timezone_offset(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::get_timezone_offset(&self.inner.clone().into(), &[JsValue::Null], context)
    }

    /// Returns the day (date) of the month (1–31) in the specified
    /// date according to universal time.
    ///
    /// Same as JavaScript's `Date.prototype.getUTCDate()`.
    #[inline]
    pub fn get_utc_date(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::get_utc_date(&self.inner.clone().into(), &[JsValue::null()], context)
    }

    /// Returns the day of the week (0–6) in the specified
    /// date according to universal time.
    ///
    /// Same as JavaScript's `Date.prototype.getUTCDay()`.
    #[inline]
    pub fn get_utc_day(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::get_utc_day(&self.inner.clone().into(), &[JsValue::null()], context)
    }

    /// Returns the year (4 digits for 4-digit years) in the specified
    /// date according to universal time.
    ///
    /// Same as JavaScript's `Date.prototype.getUTCFullYear()`.
    #[inline]
    pub fn get_utc_full_year(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::get_utc_full_year(&self.inner.clone().into(), &[JsValue::null()], context)
    }

    /// Returns the hours (0–23) in the specified date according
    /// to universal time.
    ///
    /// Same as JavaScript's `Date.prototype.getUTCHours()`.
    #[inline]
    pub fn get_utc_hours(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::get_utc_hours(&self.inner.clone().into(), &[JsValue::null()], context)
    }

    /// Returns the milliseconds (0–999) in the specified date
    /// according to universal time.
    ///
    /// Same as JavaScript's `Date.prototype.getUTCMilliseconds()`.
    #[inline]
    pub fn get_utc_milliseconds(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::get_utc_milliseconds(&self.inner.clone().into(), &[JsValue::null()], context)
    }

    /// Returns the minutes (0–59) in the specified date according
    /// to universal time.
    ///
    /// Same as JavaScript's `Date.prototype.getUTCMinutes()`.
    #[inline]
    pub fn get_utc_minutes(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::get_utc_minutes(&self.inner.clone().into(), &[JsValue::null()], context)
    }

    /// Returns the month (0–11) in the specified date according
    /// to universal time.
    ///
    /// Same as JavaScript's `Date.prototype.getUTCMonth()`.
    #[inline]
    pub fn get_utc_month(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::get_utc_month(&self.inner.clone().into(), &[JsValue::null()], context)
    }

    /// Returns the seconds (0–59) in the specified date according
    /// to universal time.
    ///
    /// Same as JavaScript's `Date.prototype.getUTCSeconds()`.
    #[inline]
    pub fn get_utc_seconds(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::get_utc_seconds(&self.inner.clone().into(), &[JsValue::null()], context)
    }

    /// DEPRECATED: This feature is no longer recommended.
    /// USE: `get_full_year()` instead.
    /// Returns the year (usually 2–3 digits) in the specified date
    /// according to local time.
    ///
    /// Same as JavaScript's `Date.prototype.getYear()`.
    #[deprecated]
    #[inline]
    pub fn get_year(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::get_year(&self.inner.clone().into(), &[JsValue::null()], context)
    }

    /// Sets the day of the month for a specified date according
    /// to local time.
    /// Takes a `month_value`.
    /// Return a `Number` representing the milliseconds elapsed between
    /// the UNIX epoch and the given date.
    ///
    /// Same as JavaScript's `Date.prototype.setDate()`.
    #[inline]
    pub fn set_date<T>(&self, value: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
    {
        Date::set_date(&self.inner.clone().into(), &[value.into()], context)
    }

    /// Sets the full year (e.g. 4 digits for 4-digit years) for a
    /// specified date according to local time.
    /// Takes [`year_value`, `month_value`, `date_value`]
    /// Return a `Number` representing the milliseconds elapsed between
    /// the UNIX epoch and updated date.
    ///
    /// Same as JavaScript's `Date.prototype.setFullYear()`.
    #[inline]
    pub fn set_full_year(&self, values: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Date::set_full_year(&self.inner.clone().into(), values, context)
    }

    /// Sets the hours for a specified date according to local time.
    /// Takes [`hours_value`, `minutes_value`, `seconds_value`, `ms_value`]
    /// Return a `Number` representing the milliseconds elapsed between
    /// the UNIX epoch and the updated date.
    ///
    /// Same as JavaScript's `Date.prototype.setHours()`.
    #[inline]
    pub fn set_hours(&self, values: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Date::set_hours(&self.inner.clone().into(), values, context)
    }

    /// Sets the milliseconds for a specified date according to local time.
    /// Takes a `milliseconds_value`
    /// Return a `Number` representing the milliseconds elapsed between
    /// the UNIX epoch and updated date.
    ///
    /// Same as JavaScript's `Date.prototype.setMilliseconds()`.
    #[inline]
    pub fn set_milliseconds<T>(&self, value: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
    {
        Date::set_milliseconds(&self.inner.clone().into(), &[value.into()], context)
    }

    /// Sets the minutes for a specified date according to local time.
    /// Takes [`minutes_value`, `seconds_value`, `ms_value`]
    /// Return a `Number` representing the milliseconds elapsed between
    /// the UNIX epoch and the updated date.
    ///
    /// Same as JavaScript's `Date.prototype.setMinutes()`.
    #[inline]
    pub fn set_minutes(&self, values: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Date::set_minutes(&self.inner.clone().into(), values, context)
    }

    /// Sets the month for a specified date according to local time.
    /// Takes [`month_value`, `day_value`]
    /// Return a `Number` representing the milliseconds elapsed between
    /// the UNIX epoch and the updated date.
    ///
    /// Same as JavaScript's `Date.prototype.setMonth()`.
    #[inline]
    pub fn set_month(&self, values: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Date::set_month(&self.inner.clone().into(), values, context)
    }

    /// Sets the seconds for a specified date according to local time.
    /// Takes [`seconds_value`, `ms_value`]
    /// Return a `Number` representing the milliseconds elapsed between
    /// the UNIX epoch and the updated date.
    ///
    /// Same as JavaScript's `Date.prototype.setSeconds()`.
    #[inline]
    pub fn set_seconds(&self, values: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Date::set_seconds(&self.inner.clone().into(), values, context)
    }

    /// Sets the Date object to the time represented by a number
    /// of milliseconds since UNIX epoch.
    /// Takes number of milliseconds since UNIX epoch.
    /// Use negative numbers for times prior.
    /// Return a `Number` representing the milliseconds elapsed between
    /// the UNIX epoch and the updated date.
    ///
    /// Same as JavaScript's `Date.prototype.setTime()`.
    #[inline]
    pub fn set_time<T>(&self, value: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
    {
        Date::set_time(&self.inner.clone().into(), &[value.into()], context)
    }

    /// Sets the day of the month for a specified date according
    /// to universal time.
    /// Takes a `month_value`.
    /// Return a `Number` representing the milliseconds elapsed between
    /// the UNIX epoch and the updated date.
    ///
    /// Same as JavaScript's `Date.prototype.setUTCDate()`.
    #[inline]
    pub fn set_utc_date<T>(&self, value: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
    {
        Date::set_utc_date(&self.inner.clone().into(), &[value.into()], context)
    }

    /// Sets the full year (e.g. 4 digits for 4-digit years) for a
    /// specified date according to universal time.
    /// Takes [`year_value`, `month_value`, `date_value`]
    /// Return a `Number` representing the milliseconds elapsed between
    /// the UNIX epoch and the updated date.
    ///
    /// Same as JavaScript's `Date.prototype.setUTCFullYear()`.
    #[inline]
    pub fn set_utc_full_year(
        &self,
        values: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Date::set_utc_full_year(&self.inner.clone().into(), values, context)
    }

    /// Sets the hours for a specified date according to universal time.
    /// Takes [`hours_value`, `minutes_value`, `seconds_value`, `ms_value`]
    /// Return a `Number` representing the milliseconds elapsed between
    /// the UNIX epoch and the updated dated.
    ///
    /// Same as JavaScript's `Date.prototype.setUTCHours()`.
    #[inline]
    pub fn set_utc_hours(&self, values: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Date::set_utc_hours(&self.inner.clone().into(), values, context)
    }

    /// Sets the milliseconds for a specified date according to universal time.
    /// Takes a `milliseconds_value`
    /// Return a `Number` representing the milliseconds elapsed between
    /// the UNIX epoch and the updated date.
    ///
    /// Same as JavaScript's `Date.prototype.setUTCMilliseconds()`.
    #[inline]
    pub fn set_utc_milliseconds<T>(&self, value: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
    {
        Date::set_utc_milliseconds(&self.inner.clone().into(), &[value.into()], context)
    }

    /// Sets the minutes for a specified date according to universal time.
    /// Takes [`minutes_value`, `seconds_value`, `ms_value`]
    /// Return a `Number` representing the milliseconds elapsed between
    /// the UNIX epoch and the updated date.
    ///
    /// Same as JavaScript's `Date.prototype.setUTCMinutes()`.
    #[inline]
    pub fn set_utc_minutes(&self, values: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Date::set_utc_minutes(&self.inner.clone().into(), values, context)
    }

    /// Sets the month for a specified date according to universal time.
    /// Takes [`month_value`, `day_value`]
    /// Return a `Number` representing the milliseconds elapsed between
    /// the UNIX epoch and the updated date.
    ///
    /// Same as JavaScript's `Date.prototype.setUTCMonth()`.
    #[inline]
    pub fn set_utc_month(&self, values: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Date::set_month(&self.inner.clone().into(), values, context)
    }

    /// Sets the seconds for a specified date according to universal time.
    /// Takes [`seconds_value`, `ms_value`]
    /// Return a `Number` representing the milliseconds elapsed between
    /// the UNIX epoch and the updated date.
    ///
    /// Same as JavaScript's `Date.prototype.setUTCSeconds()`.
    #[inline]
    pub fn set_utc_seconds(&self, values: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Date::set_utc_seconds(&self.inner.clone().into(), values, context)
    }

    /// DEPRECATED: This feature is no longer recommended.
    /// USE: `set_full_year()` instead.
    /// Sets the year for a specified date according to local time.
    /// Return a `Number` representing the milliseconds elapsed since
    /// the UNIX epoch.
    ///
    /// Same as JavaScript's legacy `Date.prototype.setYear()`.
    #[deprecated]
    #[inline]
    pub fn set_year<T>(&self, value: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
    {
        Date::set_year(&self.inner.clone().into(), &[value.into()], context)
    }

    /// Returns the "date" portion of the Date as a human-readable string.
    ///
    /// Same as JavaScript's `Date.prototype.toDateString()`.
    #[inline]
    pub fn to_date_string(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::to_date_string(&self.inner.clone().into(), &[JsValue::Null], context)
    }

    /// DEPRECATED: This feature is no longer recommended.
    /// USE: `to_utc_string()` instead.
    /// Returns a string representing the Date based on the GMT timezone.
    ///
    /// Same as JavaScript's legacy `Date.prototype.toGMTString()`
    #[deprecated]
    #[inline]
    pub fn to_gmt_string(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::to_gmt_string(&self.inner.clone().into(), &[JsValue::Null], context)
    }

    /// Returns the given date in the ISO 8601 format according to universal
    /// time.
    ///
    /// Same as JavaScript's `Date.prototype.toISOString()`.
    #[inline]
    pub fn to_iso_string(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::to_iso_string(&self.inner.clone().into(), &[JsValue::Null], context)
    }

    /// Returns a string representing the Date using `to_iso_string()`.
    ///
    /// Same as JavaScript's `Date.prototype.toJSON()`.
    #[inline]
    pub fn to_json(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::to_json(&self.inner.clone().into(), &[JsValue::Null], context)
    }

    /// Returns a string representing the date portion of the given Date instance
    /// according to language-specific conventions.
    /// Takes [locales, options]
    ///
    /// Same as JavaScript's `Date.prototype.toLocaleDateString()`.
    #[inline]
    pub fn to_local_date_string(
        &self,
        values: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Date::to_locale_date_string(&self.inner.clone().into(), values, context)
    }

    /// Returns a string representing the given date according to language-specific conventions.
    /// Takes [locales, options]
    ///
    /// Same as JavaScript's `Date.prototype.toLocaleDateString()`.
    #[inline]
    pub fn to_locale_string(&self, values: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Date::to_locale_string(&self.inner.clone().into(), values, context)
    }

    /// Returns the "time" portion of the Date as human-readable string.
    ///
    /// Same as JavaScript's `Date.prototype.toTimeString()`.
    #[inline]
    pub fn to_locale_time_string(
        &self,
        values: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Date::to_locale_time_string(&self.inner.clone().into(), values, context)
    }

    /// Returns a string representing the specified Date object.
    ///
    /// Same as JavaScript's `Date.prototype.toString()`.
    #[inline]
    pub fn to_string(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::to_string(&self.inner.clone().into(), &[JsValue::Null], context)
    }

    /// Returns the "time" portion of the Date as human-readable string.
    ///
    /// Same as JavaScript's `Date.prototype.toTimeString()`.
    #[inline]
    pub fn to_time_string(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::to_time_string(&self.inner.clone().into(), &[JsValue::Null], context)
    }

    /// Returns a string representing the given date using the UTC time zone.
    ///
    /// Same as JavaScript's `Date.prototype.toUTCString()`.
    #[inline]
    pub fn to_utc_string(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::to_utc_string(&self.inner.clone().into(), &[JsValue::Null], context)
    }

    /// Returns the primitive value pf Date object.
    ///
    /// Same as JavaScript's `Date.prototype.valueOf()`.
    #[inline]
    pub fn value_of(&self, context: &mut Context) -> JsResult<JsValue> {
        Date::value_of(&self.inner.clone().into(), &[JsValue::Null], context)
    }

    /// Utility create a `Date` object from RFC3339 string
    #[inline]
    pub fn new_from_parse(value: &JsValue, context: &mut Context) -> Self {
        let inner = Date::create_obj(value, context);
        Self { inner }
    }
}

impl From<JsDate> for JsObject {
    #[inline]
    fn from(o: JsDate) -> Self {
        o.inner.clone()
    }
}

impl From<JsDate> for JsValue {
    #[inline]
    fn from(o: JsDate) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsDate {
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl JsObjectType for JsDate {}
