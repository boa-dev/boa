//! Boa's implementation of a user-defined Anonymous Calendar.

use crate::{
    builtins::temporal::{plain_date, plain_month_day, plain_year_month},
    property::PropertyKey,
    Context, JsObject, JsString, JsValue,
};
use std::any::Any;

use boa_macros::utf16;
use boa_temporal::{
    components::{
        calendar::{CalendarDateLike, CalendarProtocol, CalendarFieldsType},
        Date, Duration, MonthDay, YearMonth,
    },
    options::ArithmeticOverflow,
    TemporalError, TemporalFields, TemporalResult, TinyAsciiStr,
};
use num_traits::ToPrimitive;
use plain_date::PlainDate;
use plain_month_day::PlainMonthDay;
use plain_year_month::PlainYearMonth;

/// A user-defined, custom calendar that is only known at runtime
/// and executed at runtime.
///
/// A user-defined calendar implements all of the `CalendarProtocolMethods`
/// and therefore satisfies the requirements to be used as a calendar.
#[derive(Debug, Clone)]
pub(crate) struct CustomRuntimeCalendar {
    calendar: JsObject,
}

impl CustomRuntimeCalendar {
    pub(crate) fn new(calendar: &JsObject) -> Self {
        Self {
            calendar: calendar.clone(),
        }
    }
}

impl CalendarProtocol for CustomRuntimeCalendar {
    fn date_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
        context: &mut dyn Any,
    ) -> TemporalResult<Date> {
        let context = context
            .downcast_mut::<Context>()
            .expect("Context was not provided for a CustomCalendar.");

        let method = self
            .calendar
            .get(utf16!("dateFromFields"), context)
            .expect("method must exist on a object that implements the CalendarProtocol.");

        let fields = JsObject::from_temporal_fields(fields, context)
            .map_err(|e| TemporalError::general(e.to_string()))?;

        let overflow_obj = JsObject::with_null_proto();

        overflow_obj
            .create_data_property_or_throw(
                utf16!("overflow"),
                JsString::from(overflow.to_string()),
                context,
            )
            .map_err(|e| TemporalError::general(e.to_string()))?;

        let value = method
            .as_callable()
            .ok_or_else(|| {
                TemporalError::general("dateFromFields must be implemented as a callable method.")
            })?
            .call(
                &self.calendar.clone().into(),
                &[fields.into(), overflow_obj.into()],
                context,
            )
            .map_err(|e| TemporalError::general(e.to_string()))?;

        let obj = value.as_object().map(JsObject::borrow).ok_or_else(|| {
            TemporalError::r#type()
                .with_message("datefromFields must return a valid PlainDate object.")
        })?;

        let pd = obj.downcast_ref::<PlainDate>().ok_or_else(|| {
            TemporalError::r#type().with_message("Object returned was not a PlainDate")
        })?;

        Ok(pd.inner.clone())
    }

    fn year_month_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
        context: &mut dyn Any,
    ) -> TemporalResult<YearMonth> {
        let context = context
            .downcast_mut::<Context>()
            .expect("Context was not provided for a CustomCalendar.");

        let method = self
            .calendar
            .get(utf16!("yearMonthFromFields"), context)
            .expect("method must exist on a object that implements the CalendarProtocol.");

        let fields = JsObject::from_temporal_fields(fields, context)
            .map_err(|e| TemporalError::general(e.to_string()))?;

        let overflow_obj = JsObject::with_null_proto();

        overflow_obj
            .create_data_property_or_throw(
                utf16!("overflow"),
                JsString::from(overflow.to_string()),
                context,
            )
            .map_err(|e| TemporalError::general(e.to_string()))?;

        let value = method
            .as_callable()
            .ok_or_else(|| {
                TemporalError::general(
                    "yearMonthFromFields must be implemented as a callable method.",
                )
            })?
            .call(
                &self.calendar.clone().into(),
                &[fields.into(), overflow_obj.into()],
                context,
            )
            .map_err(|e| TemporalError::general(e.to_string()))?;

        let obj = value.as_object().map(JsObject::borrow).ok_or_else(|| {
            TemporalError::r#type()
                .with_message("yearMonthFromFields must return a valid PlainYearMonth object.")
        })?;

        let ym = obj.downcast_ref::<PlainYearMonth>().ok_or_else(|| {
            TemporalError::r#type().with_message("Object returned was not a PlainDate")
        })?;

        Ok(ym.inner.clone())
    }

    fn month_day_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
        context: &mut dyn Any,
    ) -> TemporalResult<MonthDay> {
        let context = context
            .downcast_mut::<Context>()
            .expect("Context was not provided for a CustomCalendar.");

        let method = self
            .calendar
            .get(utf16!("yearMonthFromFields"), context)
            .expect("method must exist on a object that implements the CalendarProtocol.");

        let fields = JsObject::from_temporal_fields(fields, context)
            .map_err(|e| TemporalError::general(e.to_string()))?;

        let overflow_obj = JsObject::with_null_proto();

        overflow_obj
            .create_data_property_or_throw(
                utf16!("overflow"),
                JsString::from(overflow.to_string()),
                context,
            )
            .map_err(|e| TemporalError::general(e.to_string()))?;

        let value = method
            .as_callable()
            .ok_or_else(|| {
                TemporalError::general(
                    "yearMonthFromFields must be implemented as a callable method.",
                )
            })?
            .call(
                &self.calendar.clone().into(),
                &[fields.into(), overflow_obj.into()],
                context,
            )
            .map_err(|e| TemporalError::general(e.to_string()))?;

        let obj = value.as_object().map(JsObject::borrow).ok_or_else(|| {
            TemporalError::r#type()
                .with_message("yearMonthFromFields must return a valid PlainYearMonth object.")
        })?;

        let md = obj.downcast_ref::<PlainMonthDay>().ok_or_else(|| {
            TemporalError::r#type().with_message("Object returned was not a PlainDate")
        })?;

        Ok(md.inner.clone())
    }

    fn date_add(
        &self,
        _date: &Date,
        _duration: &Duration,
        _overflow: ArithmeticOverflow,
        _context: &mut dyn Any,
    ) -> TemporalResult<Date> {
        // TODO
        Err(TemporalError::general("Not yet implemented."))
    }

    fn date_until(
        &self,
        _one: &Date,
        _two: &Date,
        _largest_unit: boa_temporal::options::TemporalUnit,
        _context: &mut dyn Any,
    ) -> TemporalResult<Duration> {
        // TODO
        Err(TemporalError::general("Not yet implemented."))
    }

    fn era(
        &self,
        _: &CalendarDateLike,
        _: &mut dyn Any,
    ) -> TemporalResult<Option<TinyAsciiStr<8>>> {
        // Return undefined as custom calendars do not implement -> Currently.
        Ok(None)
    }

    fn era_year(&self, _: &CalendarDateLike, _: &mut dyn Any) -> TemporalResult<Option<i32>> {
        // Return undefined as custom calendars do not implement -> Currently.
        Ok(None)
    }

    fn year(&self, date_like: &CalendarDateLike, context: &mut dyn Any) -> TemporalResult<i32> {
        let context = context
            .downcast_mut::<Context>()
            .expect("Context was not provided for a CustomCalendar.");

        let date_like = date_like_to_object(date_like, context)?;

        let method = self
            .calendar
            .get(PropertyKey::from(utf16!("year")), context)
            .expect("method must exist on a object that implements the CalendarProtocol.");

        let val = method
            .as_callable()
            .expect("is method")
            .call(&method, &[date_like], context)
            .map_err(|err| TemporalError::general(err.to_string()))?;

        // Validate the return value.
        // 3. If Type(result) is not Number, throw a TypeError exception.
        // 4. If IsIntegralNumber(result) is false, throw a RangeError exception.
        // 5. If result < 1𝔽, throw a RangeError exception.
        // 6. Return ℝ(result).

        let Some(number) = val.as_number() else {
            return Err(TemporalError::r#type().with_message("year must return a number."));
        };

        if !number.is_finite() || number.fract() != 0.0 {
            return Err(TemporalError::r#type().with_message("year return must be integral."));
        }

        if number < 1f64 {
            return Err(TemporalError::r#type().with_message("year return must be larger than 1."));
        }

        let result = number
            .to_i32()
            .ok_or_else(|| TemporalError::range().with_message("year exceeded a valid range."))?;

        Ok(result)
    }

    fn month(&self, date_like: &CalendarDateLike, context: &mut dyn Any) -> TemporalResult<u8> {
        let context = context
            .downcast_mut::<Context>()
            .expect("Context was not provided for a CustomCalendar.");

        let date_like = date_like_to_object(date_like, context)?;

        let method = self
            .calendar
            .get(PropertyKey::from(utf16!("month")), context)
            .expect("method must exist on a object that implements the CalendarProtocol.");

        let val = method
            .as_callable()
            .expect("is method")
            .call(&method, &[date_like], context)
            .map_err(|err| TemporalError::general(err.to_string()))?;

        // Validate the return value.
        // 3. If Type(result) is not Number, throw a TypeError exception.
        // 4. If IsIntegralNumber(result) is false, throw a RangeError exception.
        // 5. If result < 1𝔽, throw a RangeError exception.
        // 6. Return ℝ(result).

        let Some(number) = val.as_number() else {
            return Err(TemporalError::r#type().with_message("month must return a number."));
        };

        if !number.is_finite() || number.fract() != 0.0 {
            return Err(TemporalError::r#type().with_message("month return must be integral."));
        }

        if number < 1f64 {
            return Err(TemporalError::r#type().with_message("month return must be larger than 1."));
        }

        let result = number
            .to_u8()
            .ok_or_else(|| TemporalError::range().with_message("month exceeded a valid range."))?;

        Ok(result)
    }

    fn month_code(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<TinyAsciiStr<4>> {
        let context = context
            .downcast_mut::<Context>()
            .expect("Context was not provided for a CustomCalendar.");

        let date_like = date_like_to_object(date_like, context)?;

        let method = self
            .calendar
            .get(PropertyKey::from(utf16!("monthCode")), context)
            .expect("method must exist on a object that implements the CalendarProtocol.");

        let val = method
            .as_callable()
            .expect("is method")
            .call(&method, &[date_like], context)
            .map_err(|err| TemporalError::general(err.to_string()))?;

        let JsValue::String(result) = val else {
            return Err(TemporalError::r#type().with_message("monthCode return must be a String."));
        };

        let result = TinyAsciiStr::<4>::from_str(&result.to_std_string_escaped())
            .map_err(|_| TemporalError::general("Unexpected monthCode value."))?;

        Ok(result)
    }

    fn day(&self, date_like: &CalendarDateLike, context: &mut dyn Any) -> TemporalResult<u8> {
        let context = context
            .downcast_mut::<Context>()
            .expect("Context was not provided for a CustomCalendar.");

        let date_like = date_like_to_object(date_like, context)?;

        let method = self
            .calendar
            .get(PropertyKey::from(utf16!("day")), context)
            .expect("method must exist on a object that implements the CalendarProtocol.");

        let val = method
            .as_callable()
            .expect("is method")
            .call(&method, &[date_like], context)
            .map_err(|err| TemporalError::general(err.to_string()))?;

        // Validate the return value.
        // 3. If Type(result) is not Number, throw a TypeError exception.
        // 4. If IsIntegralNumber(result) is false, throw a RangeError exception.
        // 5. If result < 1𝔽, throw a RangeError exception.
        // 6. Return ℝ(result).

        let Some(number) = val.as_number() else {
            return Err(TemporalError::r#type().with_message("day must return a number."));
        };

        if !number.is_finite() || number.fract() != 0.0 {
            return Err(TemporalError::r#type().with_message("day return must be integral."));
        }

        if number < 1f64 {
            return Err(TemporalError::r#type().with_message("day return must be larger than 1."));
        }

        let result = number
            .to_u8()
            .ok_or_else(|| TemporalError::range().with_message("day exceeded a valid range."))?;

        Ok(result)
    }

    fn day_of_week(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<u16> {
        let context = context
            .downcast_mut::<Context>()
            .expect("Context was not provided for a CustomCalendar.");

        let date_like = date_like_to_object(date_like, context)?;

        let method = self
            .calendar
            .get(PropertyKey::from(utf16!("dayOfWeek")), context)
            .expect("method must exist on a object that implements the CalendarProtocol.");

        let val = method
            .as_callable()
            .expect("is method")
            .call(&method, &[date_like], context)
            .map_err(|err| TemporalError::general(err.to_string()))?;

        // Validate the return value.
        // 3. If Type(result) is not Number, throw a TypeError exception.
        // 4. If IsIntegralNumber(result) is false, throw a RangeError exception.
        // 5. If result < 1𝔽, throw a RangeError exception.
        // 6. Return ℝ(result).

        let Some(number) = val.as_number() else {
            return Err(TemporalError::r#type().with_message("DayOfWeek must return a number."));
        };

        if !number.is_finite() || number.fract() != 0.0 {
            return Err(TemporalError::r#type().with_message("DayOfWeek return must be integral."));
        }

        if number < 1f64 {
            return Err(
                TemporalError::r#type().with_message("DayOfWeek return must be larger than 1.")
            );
        }

        let result = number.to_u16().ok_or_else(|| {
            TemporalError::range().with_message("DayOfWeek exceeded valid range.")
        })?;

        Ok(result)
    }

    fn day_of_year(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<u16> {
        let context = context
            .downcast_mut::<Context>()
            .expect("Context was not provided for a CustomCalendar.");

        let date_like = date_like_to_object(date_like, context)?;

        let method = self
            .calendar
            .get(PropertyKey::from(utf16!("dayOfYear")), context)
            .expect("method must exist on a object that implements the CalendarProtocol.");

        let val = method
            .as_callable()
            .expect("is method")
            .call(&method, &[date_like], context)
            .map_err(|err| TemporalError::general(err.to_string()))?;

        // Validate the return value.
        // 3. If Type(result) is not Number, throw a TypeError exception.
        // 4. If IsIntegralNumber(result) is false, throw a RangeError exception.
        // 5. If result < 1𝔽, throw a RangeError exception.
        // 6. Return ℝ(result).

        let Some(number) = val.as_number() else {
            return Err(TemporalError::r#type().with_message("dayOfYear must return a number."));
        };

        if !number.is_finite() || number.fract() != 0.0 {
            return Err(TemporalError::r#type().with_message("dayOfYear return must be integral."));
        }

        if number < 1f64 {
            return Err(
                TemporalError::r#type().with_message("dayOfYear return must be larger than 1.")
            );
        }

        let result = number.to_u16().ok_or_else(|| {
            TemporalError::range().with_message("dayOfYear exceeded valid range.")
        })?;

        Ok(result)
    }

    fn week_of_year(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<u16> {
        let context = context
            .downcast_mut::<Context>()
            .expect("Context was not provided for a CustomCalendar.");

        let date_like = date_like_to_object(date_like, context)?;

        let method = self
            .calendar
            .get(PropertyKey::from(utf16!("weekOfYear")), context)
            .expect("method must exist on a object that implements the CalendarProtocol.");

        let val = method
            .as_callable()
            .expect("is method")
            .call(&method, &[date_like], context)
            .map_err(|err| TemporalError::general(err.to_string()))?;

        // Validate the return value.
        // 3. If Type(result) is not Number, throw a TypeError exception.
        // 4. If IsIntegralNumber(result) is false, throw a RangeError exception.
        // 5. If result < 1𝔽, throw a RangeError exception.
        // 6. Return ℝ(result).

        let Some(number) = val.as_number() else {
            return Err(TemporalError::r#type().with_message("weekOfYear must return a number."));
        };

        if !number.is_finite() || number.fract() != 0.0 {
            return Err(TemporalError::r#type().with_message("weekOfYear return must be integral."));
        }

        if number < 1f64 {
            return Err(
                TemporalError::r#type().with_message("weekOfYear return must be larger than 1.")
            );
        }

        let result = number.to_u16().ok_or_else(|| {
            TemporalError::range().with_message("weekOfYear exceeded valid range.")
        })?;

        Ok(result)
    }

    fn year_of_week(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<i32> {
        let context = context
            .downcast_mut::<Context>()
            .expect("Context was not provided for a CustomCalendar.");

        let date_like = date_like_to_object(date_like, context)?;

        let method = self
            .calendar
            .get(PropertyKey::from(utf16!("yearOfWeek")), context)
            .expect("method must exist on a object that implements the CalendarProtocol.");

        let val = method
            .as_callable()
            .expect("is method")
            .call(&method, &[date_like], context)
            .map_err(|err| TemporalError::general(err.to_string()))?;

        // Validate the return value.
        // 3. If Type(result) is not Number, throw a TypeError exception.
        // 4. If IsIntegralNumber(result) is false, throw a RangeError exception.
        // 5. Return ℝ(result).

        let Some(number) = val.as_number() else {
            return Err(TemporalError::r#type().with_message("yearOfWeek must return a number."));
        };

        if !number.is_finite() || number.fract() != 0.0 {
            return Err(TemporalError::r#type().with_message("yearOfWeek return must be integral."));
        }

        let result = number.to_i32().ok_or_else(|| {
            TemporalError::range().with_message("yearOfWeek exceeded valid range.")
        })?;

        Ok(result)
    }

    fn days_in_week(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<u16> {
        let context = context
            .downcast_mut::<Context>()
            .expect("Context was not provided for a CustomCalendar.");

        let date_like = date_like_to_object(date_like, context)?;

        let method = self
            .calendar
            .get(PropertyKey::from(utf16!("daysInWeek")), context)
            .expect("method must exist on a object that implements the CalendarProtocol.");

        let val = method
            .as_callable()
            .expect("is method")
            .call(&method, &[date_like], context)
            .map_err(|err| TemporalError::general(err.to_string()))?;

        // Validate the return value.
        // 3. If Type(result) is not Number, throw a TypeError exception.
        // 4. If IsIntegralNumber(result) is false, throw a RangeError exception.
        // 5. If result < 1𝔽, throw a RangeError exception.
        // 6. Return ℝ(result).

        let Some(number) = val.as_number() else {
            return Err(TemporalError::r#type().with_message("daysInWeek must return a number."));
        };

        if !number.is_finite() || number.fract() != 0.0 {
            return Err(TemporalError::r#type().with_message("daysInWeek return must be integral."));
        }

        if number < 1f64 {
            return Err(
                TemporalError::r#type().with_message("daysInWeek return must be larger than 1.")
            );
        }

        let result = number.to_u16().ok_or_else(|| {
            TemporalError::range().with_message("daysInWeek exceeded valid range.")
        })?;

        Ok(result)
    }

    fn days_in_month(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<u16> {
        let context = context
            .downcast_mut::<Context>()
            .expect("Context was not provided for a CustomCalendar.");

        let date_like = date_like_to_object(date_like, context)?;

        let method = self
            .calendar
            .get(PropertyKey::from(utf16!("daysInMonth")), context)
            .expect("method must exist on a object that implements the CalendarProtocol.");
        let val = method
            .as_callable()
            .expect("is method")
            .call(&method, &[date_like], context)
            .map_err(|err| TemporalError::general(err.to_string()))?;

        // Validate the return value.
        // 3. If Type(result) is not Number, throw a TypeError exception.
        // 4. If IsIntegralNumber(result) is false, throw a RangeError exception.
        // 5. If result < 1𝔽, throw a RangeError exception.
        // 6. Return ℝ(result).

        let Some(number) = val.as_number() else {
            return Err(TemporalError::r#type().with_message("daysInMonth must return a number."));
        };

        if !number.is_finite() || number.fract() != 0.0 {
            return Err(
                TemporalError::r#type().with_message("daysInMonth return must be integral.")
            );
        }

        if number < 1f64 {
            return Err(
                TemporalError::r#type().with_message("daysInMonth return must be larger than 1.")
            );
        }

        let result = number.to_u16().ok_or_else(|| {
            TemporalError::range().with_message("daysInMonth exceeded valid range.")
        })?;

        Ok(result)
    }

    fn days_in_year(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<u16> {
        let context = context
            .downcast_mut::<Context>()
            .expect("Context was not provided for a CustomCalendar.");

        let date_like = date_like_to_object(date_like, context)?;

        let method = self
            .calendar
            .get(PropertyKey::from(utf16!("daysInYear")), context)
            .expect("method must exist on a object that implements the CalendarProtocol.");

        let val = method
            .as_callable()
            .expect("is method")
            .call(&method, &[date_like], context)
            .map_err(|err| TemporalError::general(err.to_string()))?;

        // Validate the return value.
        // 3. If Type(result) is not Number, throw a TypeError exception.
        // 4. If IsIntegralNumber(result) is false, throw a RangeError exception.
        // 5. If result < 1𝔽, throw a RangeError exception.
        // 6. Return ℝ(result).

        let Some(number) = val.as_number() else {
            return Err(TemporalError::r#type().with_message("daysInYear must return a number."));
        };

        if !number.is_finite() || number.fract() != 0.0 {
            return Err(TemporalError::r#type().with_message("daysInYear return must be integral."));
        }

        if number < 1f64 {
            return Err(
                TemporalError::r#type().with_message("daysInYear return must be larger than 1.")
            );
        }

        let result = number.to_u16().ok_or_else(|| {
            TemporalError::range().with_message("monthsInYear exceeded valid range.")
        })?;

        Ok(result)
    }

    fn months_in_year(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<u16> {
        let context = context
            .downcast_mut::<Context>()
            .expect("Context was not provided for a CustomCalendar.");

        let date_like = date_like_to_object(date_like, context)?;

        let method = self
            .calendar
            .get(PropertyKey::from(utf16!("monthsInYear")), context)
            .expect("method must exist on a object that implements the CalendarProtocol.");

        let val = method
            .as_callable()
            .expect("is method")
            .call(&method, &[date_like], context)
            .map_err(|err| TemporalError::general(err.to_string()))?;

        // Validate the return value.
        // 3. If Type(result) is not Number, throw a TypeError exception.
        // 4. If IsIntegralNumber(result) is false, throw a RangeError exception.
        // 5. If result < 1𝔽, throw a RangeError exception.
        // 6. Return ℝ(result).

        let Some(number) = val.as_number() else {
            return Err(TemporalError::r#type().with_message("monthsInYear must return a number."));
        };

        if !number.is_finite() || number.fract() != 0.0 {
            return Err(
                TemporalError::r#type().with_message("monthsInYear return must be integral.")
            );
        }

        if number < 1f64 {
            return Err(
                TemporalError::r#type().with_message("monthsInYear return must be larger than 1.")
            );
        }

        let result = number.to_u16().ok_or_else(|| {
            TemporalError::range().with_message("monthsInYear exceeded valid range.")
        })?;

        Ok(result)
    }

    fn in_leap_year(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<bool> {
        let context = context
            .downcast_mut::<Context>()
            .expect("Context was not provided for a CustomCalendar.");

        let date_like = date_like_to_object(date_like, context)?;

        let method = self
            .calendar
            .get(PropertyKey::from(utf16!("inLeapYear")), context)
            .expect("method must exist on a object that implements the CalendarProtocol.");

        let val = method
            .as_callable()
            .expect("is method")
            .call(&method, &[date_like], context)
            .map_err(|err| TemporalError::general(err.to_string()))?;

        let JsValue::Boolean(result) = val else {
            return Err(
                TemporalError::r#type().with_message("inLeapYear must return a valid boolean.")
            );
        };

        Ok(result)
    }

    // TODO: Determine fate of fn fields()

    fn field_descriptors(
        &self,
        _: CalendarFieldsType,
    ) -> Vec<(String, bool)> {
        Vec::default()
    }

    fn field_keys_to_ignore(&self, _: Vec<String>) -> Vec<String> {
        Vec::default()
    }

    fn resolve_fields(
        &self,
        _: &mut TemporalFields,
        _: CalendarFieldsType,
    ) -> TemporalResult<()> {
        Ok(())
    }

    fn identifier(&self, context: &mut dyn Any) -> TemporalResult<String> {
        let context = context
            .downcast_mut::<Context>()
            .expect("Context was not provided for a CustomCalendar.");

        let identifier = self
            .calendar
            .__get__(
                &PropertyKey::from(utf16!("id")),
                JsValue::undefined(),
                &mut context.into(),
            )
            .expect("method must exist on a object that implements the CalendarProtocol.");

        let JsValue::String(s) = identifier else {
            return Err(TemporalError::range().with_message("Identifier was not a string"));
        };

        Ok(s.to_std_string_escaped())
    }
}

/// Utility function for converting `Temporal`'s `CalendarDateLike` to it's `Boa` specific `JsObject`.
pub(crate) fn date_like_to_object(
    date_like: &CalendarDateLike,
    context: &mut Context,
) -> TemporalResult<JsValue> {
    match date_like {
        CalendarDateLike::Date(d) => plain_date::create_temporal_date(d.clone(), None, context)
            .map_err(|e| TemporalError::general(e.to_string()))
            .map(Into::into),
        CalendarDateLike::DateTime(_dt) => {
            todo!()
        }
        CalendarDateLike::MonthDay(md) => {
            plain_month_day::create_temporal_month_day(md.clone(), None, context)
                .map_err(|e| TemporalError::general(e.to_string()))
        }
        CalendarDateLike::YearMonth(ym) => {
            plain_year_month::create_temporal_year_month(ym.clone(), None, context)
                .map_err(|e| TemporalError::general(e.to_string()))
        }
    }
}
