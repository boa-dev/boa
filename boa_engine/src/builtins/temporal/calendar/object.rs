//! Boa's implementation of a user-defined Anonymous Calendar.

use crate::{
    builtins::temporal::{plain_date, plain_month_day, plain_year_month},
    object::ObjectKind,
    property::PropertyKey,
    Context, JsObject, JsValue,
};
use std::any::Any;

use boa_macros::utf16;
use boa_temporal::{
    calendar::{CalendarDateLike, CalendarProtocol},
    date::TemporalDate,
    duration::Duration,
    error::TemporalError,
    fields::TemporalFields,
    month_day::TemporalMonthDay,
    options::ArithmeticOverflow,
    year_month::TemporalYearMonth,
    TemporalResult, TinyStr4, TinyStr8,
};

/// A user-defined, custom calendar that is only known at runtime
/// and executed at runtime.
///
/// A user-defined calendar implements all methods of the CalendarProtocol,
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
    ) -> TemporalResult<TemporalDate> {
        // Safety: Context lives for the life of the program and execution, so
        // this should, in theory, be valid.
        let context = context
            .downcast_mut::<Context>()
            .expect("Context was not provided for a CustomCalendar.");

        let method = self
            .calendar
            .get(utf16!("dateFromFields"), context)
            .expect("method must exist on a object that implements the CalendarProtocol.");

        let fields = JsObject::from_temporal_fields(&fields, context)
            .map_err(|_| TemporalError::range().with_message("Need new"))?;

        let value = method
            .call(&self.calendar.clone().into(), &[fields.into()], context)
            .map_err(|e| {
                TemporalError::range().with_message("Update error to handle error conversions")
            })?;

        let obj = value
            .as_object()
            .map(JsObject::borrow)
            .ok_or_else(|| TemporalError::r#type().with_message("could not borrow object"))?;

        let ObjectKind::PlainDate(pd) = obj.kind() else {
            return Err(TemporalError::r#type().with_message("Object returned was not a PlainDate"));
        };

        Ok(pd.inner.clone())
    }

    fn year_month_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
        context: &mut dyn Any,
    ) -> TemporalResult<TemporalYearMonth> {
        todo!()
    }

    fn month_day_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
        context: &mut dyn Any,
    ) -> TemporalResult<TemporalMonthDay> {
        todo!()
    }

    fn date_add(
        &self,
        date: &TemporalDate,
        duration: &Duration,
        overflow: ArithmeticOverflow,
        context: &mut dyn Any,
    ) -> TemporalResult<TemporalDate> {
        todo!()
    }

    fn date_until(
        &self,
        one: &TemporalDate,
        two: &TemporalDate,
        largest_unit: boa_temporal::options::TemporalUnit,
        context: &mut dyn Any,
    ) -> TemporalResult<Duration> {
        todo!()
    }

    // TODO: Determine validity of below errors.
    fn era(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<Option<TinyStr8>> {
        Err(TemporalError::range().with_message("Objects do not implement era"))
    }

    fn era_year(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<Option<i32>> {
        Err(TemporalError::range().with_message("Objects do not implement eraYear."))
    }

    fn year(&self, date_like: &CalendarDateLike, context: &mut dyn Any) -> TemporalResult<i32> {
        todo!()
    }

    fn month(&self, date_like: &CalendarDateLike, context: &mut dyn Any) -> TemporalResult<u8> {
        todo!()
    }

    fn month_code(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<TinyStr4> {
        todo!()
    }

    fn day(&self, date_like: &CalendarDateLike, context: &mut dyn Any) -> TemporalResult<u8> {
        todo!()
    }

    fn day_of_week(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<i32> {
        todo!()
    }

    fn day_of_year(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<i32> {
        todo!()
    }

    fn week_of_year(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<i32> {
        todo!()
    }

    fn year_of_week(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<i32> {
        todo!()
    }

    fn days_in_week(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<i32> {
        // Safety: Context lives for the lifetime of the program's execution, so
        // this should, in theory, be safe memory to access.
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

        let JsValue::Integer(integral) = val else {
            return Err(TemporalError::range().with_message("Invalid CustomCalendarReturn"));
        };

        Ok(integral)
    }

    fn days_in_month(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<i32> {
        // Safety: Context lives for the lifetime of the program's execution, so
        // this should, in theory, be safe memory to access.
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

        let JsValue::Integer(integral) = val else {
            return Err(TemporalError::range().with_message("Invalid CustomCalendarReturn"));
        };

        Ok(integral)
    }

    fn days_in_year(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<i32> {
        // Safety: Context lives for the lifetime of the program's execution, so
        // this should, in theory, be safe memory to access.
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

        let JsValue::Integer(integral) = val else {
            return Err(TemporalError::range().with_message("Invalid CustomCalendarReturn"));
        };

        Ok(integral)
    }

    fn months_in_year(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<i32> {
        todo!()
    }

    fn in_leap_year(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<bool> {
        todo!()
    }

    // TODO: Determine fate of fn fields()

    fn field_descriptors(
        &self,
        r#type: boa_temporal::calendar::CalendarFieldsType,
    ) -> Vec<(String, bool)> {
        Vec::default()
    }

    fn field_keys_to_ignore(&self, additional_keys: Vec<String>) -> Vec<String> {
        Vec::default()
    }

    fn resolve_fields(
        &self,
        fields: &mut TemporalFields,
        r#type: boa_temporal::calendar::CalendarFieldsType,
    ) -> TemporalResult<()> {
        todo!()
    }

    fn identifier(&self, context: &mut dyn Any) -> TemporalResult<String> {
        // Safety: Context lives for the lifetime of the program's execution, so
        // this should, in theory, be safe memory to access.
        let context = context
            .downcast_mut::<Context>()
            .expect("Context was not provided for a CustomCalendar.");

        let identifier = self
            .calendar
            .__get__(
                &PropertyKey::from(utf16!("id")),
                JsValue::undefined(),
                context,
            )
            .expect("method must exist on a object that implements the CalendarProtocol.");

        let JsValue::String(s) = identifier else {
            return Err(TemporalError::range().with_message("Identifier was not a string"));
        };

        Ok(s.to_std_string_escaped())
    }
}

pub(crate) fn date_like_to_object(
    date_like: &CalendarDateLike,
    context: &mut Context,
) -> TemporalResult<JsValue> {
    match date_like {
        CalendarDateLike::Date(d) => plain_date::create_temporal_date(d.clone(), None, context)
            .map_err(|e| TemporalError::general(e.to_string()))
            .map(Into::into),
        CalendarDateLike::DateTime(dt) => {
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
