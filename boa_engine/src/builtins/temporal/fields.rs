use crate::{
    js_string, property::PropertyKey, string::utf16, value::PreferredType, Context, JsNativeError,
    JsObject, JsResult, JsString, JsValue,
};

use super::plain_date::iso::IsoDateRecord;

use rustc_hash::FxHashSet;

/// The temproal fields are laid out in the Temporal proposal under section 13.46 `PrepareTemporalFields`
/// with conversion and defaults laid out by Table 17 (displayed below).
///
/// ## Table 17: Temporal field requirements
/// | -------------|---------------------------------|------------|
/// |   Property   |           Conversion            |  Default   |
/// | -------------|---------------------------------|------------|
/// | "year"	   |     ToIntegerWithTruncation     | undefined  |
/// | -------------|---------------------------------|------------|
/// | "month"	   | ToPositiveIntegerWithTruncation | undefined  |
/// | -------------|---------------------------------|------------|
/// | "monthCode"  |   ToPrimitiveAndRequireString   | undefined  |
/// | -------------|---------------------------------|------------|
/// | "day"        | ToPositiveIntegerWithTruncation | undefined  |
/// | -------------|---------------------------------|------------|
/// | "hour"       |     ToIntegerWithTruncation     |    +0𝔽     |
/// | -------------|---------------------------------|------------|
/// | "minute"	   |     ToIntegerWithTruncation     |    +0𝔽     |
/// | -------------|---------------------------------|------------|
/// | "second"     |     ToIntegerWithTruncation	 |    +0𝔽     |
/// | -------------|---------------------------------|------------|
/// | "millisecond"|     ToIntegerWithTruncation     |    +0𝔽     |
/// | -------------|---------------------------------|------------|
/// | "microsecond"|     ToIntegerWithTruncation     |    +0𝔽     |
/// | -------------|---------------------------------|------------|
/// | "nanosecond" |     ToIntegerWithTruncation     |    +0𝔽     |
/// | -------------|---------------------------------|------------|
/// | "offset"     |   ToPrimitiveAndRequireString   | undefined  |
/// | -------------|---------------------------------|------------|
/// | "era"        |   ToPrimitiveAndRequireString   | undefined  |
/// | -------------|---------------------------------|------------|
/// | "eraYear"    |     ToIntegerWithTruncation     | undefined  |
/// | -------------|---------------------------------|------------|
/// | "timeZone"   |                                 | undefined  |
/// |-------------------------------------------------------------|
///
/// `TemporalFields` acts as a middle ground between Table 17 and a native Rust
/// implementation of the fields.
pub(crate) struct TemporalFields {
    year: Option<i32>,
    month: Option<i32>,
    month_code: Option<JsString>, // TODO: Switch to icu compatible value.
    day: Option<i32>,
    hour: i32,
    minute: i32,
    second: i32,
    millisecond: i32,
    microsecond: i32,
    nanosecond: i32,
    offset: Option<JsString>,
    era: Option<JsString>,       // TODO: switch to icu compatible value.
    era_year: Option<JsString>,  // TODO: switch to icu compatible value.
    time_zone: Option<JsString>, // TODO: figure out the identifier for TimeZone.
}

impl Default for TemporalFields {
    fn default() -> Self {
        Self {
            year: None,
            month: None,
            month_code: None,
            day: None,
            hour: 0,
            minute: 0,
            second: 0,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
            offset: None,
            era: None,
            era_year: None,
            time_zone: None,
        }
    }
}

impl TemporalFields {
    pub(crate) fn year(&self) -> Option<i32> {
        self.year
    }

    pub(crate) fn month(&self) -> Option<i32> {
        self.month
    }

    pub(crate) fn day(&self) -> Option<i32> {
        self.day
    }
}

impl TemporalFields {
    #[inline]
    fn set_field_value(
        &mut self,
        field: &JsString,
        value: &JsValue,
        context: &mut Context<'_>,
    ) -> JsResult<()> {
        match field.as_ref() {
            super::YEAR => self.set_year(value, context)?,
            super::MONTH => self.set_month(value, context)?,
            super::MONTH_CODE => self.set_month_code(value, context)?,
            super::DAY => self.set_day(value, context)?,
            super::HOUR => self.set_hour(value, context)?,
            super::MINUTE => self.set_minute(value, context)?,
            super::SECOND => self.set_second(value, context)?,
            super::MILLISECOND => self.set_milli(value, context)?,
            super::MICROSECOND => self.set_micro(value, context)?,
            super::NANOSECOND => self.set_nano(value, context)?,
            super::OFFSET => self.set_nano(value, context)?,
            super::ERA => self.set_era(value, context)?,
            super::ERA_YEAR => self.set_era_year(value, context)?,
            super::TZ => self.set_time_zone(value)?,
            _ => unreachable!(),
        }

        Ok(())
    }

    #[inline]
    fn set_year(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let y = super::to_integer_with_truncation(value, context)?;
        self.year = Some(y);
        Ok(())
    }

    #[inline]
    fn set_month(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let mo = super::to_positive_integer_with_trunc(value, context)?;
        self.year = Some(mo);
        Ok(())
    }

    #[inline]
    fn set_month_code(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let mc = value.to_primitive(context, PreferredType::String)?;
        if let Some(string) = mc.as_string() {
            self.month_code = Some(string.clone())
        } else {
            return Err(JsNativeError::typ()
                .with_message("ToPrimativeAndRequireString must be of type String.")
                .into());
        }

        Ok(())
    }

    #[inline]
    fn set_day(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let d = super::to_positive_integer_with_trunc(value, context)?;
        self.day = Some(d);
        Ok(())
    }

    #[inline]
    fn set_hour(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let h = super::to_integer_with_truncation(value, context)?;
        self.hour = h;
        Ok(())
    }

    #[inline]
    fn set_minute(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let m = super::to_integer_with_truncation(value, context)?;
        self.minute = m;
        Ok(())
    }

    #[inline]
    fn set_second(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let sec = super::to_integer_with_truncation(value, context)?;
        self.second = sec;
        Ok(())
    }

    #[inline]
    fn set_milli(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let milli = super::to_integer_with_truncation(value, context)?;
        self.millisecond = milli;
        Ok(())
    }

    #[inline]
    fn set_micro(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let micro = super::to_integer_with_truncation(value, context)?;
        self.microsecond = micro;
        Ok(())
    }

    #[inline]
    fn set_nano(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let nano = super::to_integer_with_truncation(value, context)?;
        self.nanosecond = nano;
        Ok(())
    }

    #[inline]
    fn set_offset(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let mc = value.to_primitive(context, PreferredType::String)?;
        if let Some(string) = mc.as_string() {
            self.month_code = Some(string.clone())
        } else {
            return Err(JsNativeError::typ()
                .with_message("ToPrimativeAndRequireString must be of type String.")
                .into());
        }

        Ok(())
    }

    #[inline]
    fn set_era(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let mc = value.to_primitive(context, PreferredType::String)?;
        if let Some(string) = mc.as_string() {
            self.month_code = Some(string.clone())
        } else {
            return Err(JsNativeError::typ()
                .with_message("ToPrimativeAndRequireString must be of type String.")
                .into());
        }

        Ok(())
    }

    #[inline]
    fn set_era_year(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let mc = value.to_primitive(context, PreferredType::String)?;
        if let Some(string) = mc.as_string() {
            self.month_code = Some(string.clone())
        } else {
            return Err(JsNativeError::typ()
                .with_message("ToPrimativeAndRequireString must be of type String.")
                .into());
        }

        Ok(())
    }

    #[inline]
    fn set_time_zone(&mut self, value: &JsValue) -> JsResult<()> {
        let tz = value.as_string().map(|s| s.clone());
        self.time_zone = tz;
        Ok(())
    }
}

impl TemporalFields {
    // NOTE: required_fields should be None when it is set to Partial.
    /// The equivalant function to PrepareTemporalFields
    pub(crate) fn from_js_object(
        fields: &JsObject,
        field_names: &[JsString],
        required_fields: Option<&[JsString]>,
        dup_behaviour: Option<JsString>,
        context: &mut Context<'_>,
    ) -> JsResult<Self> {
        // 1. If duplicateBehaviour is not present, set duplicateBehaviour to throw.
        let dup_option = dup_behaviour.unwrap_or(js_string!("throw"));

        // 2. Let result be OrdinaryObjectCreate(null).
        let mut result = Self::default();

        // 3. Let any be false.
        let mut any = false;
        // 4. Let sortedFieldNames be SortStringListByCodeUnit(fieldNames).
        // 5. Let previousProperty be undefined.
        let mut dups_map = FxHashSet::default();

        // 6. For each property name property of sortedFieldNames, do
        for field in field_names {
            // a. If property is one of "constructor" or "__proto__", then
            if field.as_ref() == utf16!("constructor") || field.as_ref() == utf16!("__proto__") {
                // i. Throw a RangeError exception.
                return Err(JsNativeError::range()
                    .with_message("constructor or proto is out of field range.")
                    .into());
            }

            // NOTE: is this safe with JsString?
            let new_value = dups_map.insert(field.to_std_string_escaped());

            // b. If property is not equal to previousProperty, then
            if new_value {
                // i. Let value be ? Get(fields, property).
                let value = fields.get(PropertyKey::from(field.as_ref()), context)?;
                // ii. If value is not undefined, then
                if !value.is_undefined() {
                    any = true;
                    // 1. Set any to true.
                    // 2. If property is in the Property column of Table 17 and there is a Conversion value in the same row, then
                    // a. Let Conversion be the Conversion value of the same row.
                    // b. If Conversion is ToIntegerWithTruncation, then
                    // i. Set value to ? ToIntegerWithTruncation(value).
                    // ii. Set value to 𝔽(value).
                    // c. Else if Conversion is ToPositiveIntegerWithTruncation, then
                    // i. Set value to ? ToPositiveIntegerWithTruncation(value).
                    // ii. Set value to 𝔽(value).
                    // d. Else,
                    // i. Assert: Conversion is ToPrimitiveAndRequireString.
                    // ii. NOTE: Non-primitive values are supported here for consistency with other fields, but such values must coerce to Strings.
                    // iii. Set value to ? ToPrimitive(value, string).
                    // iv. If value is not a String, throw a TypeError exception.
                    // 3. Perform ! CreateDataPropertyOrThrow(result, property, value).
                    result.set_field_value(field, &value, context)?
                // iii. Else if requiredFields is a List, then
                } else if let Some(list) = required_fields {
                    // 1. If requiredFields contains property, then
                    if list.contains(&field) {
                        // a. Throw a TypeError exception.
                        return Err(JsNativeError::typ()
                            .with_message("A required temporal field was not provided.")
                            .into());
                    }

                    // NOTE: Values set to a default on init.
                    // 2. If property is in the Property column of Table 17, then
                    // a. Set value to the corresponding Default value of the same row.
                    // 3. Perform ! CreateDataPropertyOrThrow(result, property, value).
                }
            // c. Else if duplicateBehaviour is throw, then
            } else if dup_option.to_std_string_escaped() == "throw" {
                // i. Throw a RangeError exception.
                return Err(JsNativeError::range()
                    .with_message("Cannot have a duplicate field")
                    .into());
            }
            // d. Set previousProperty to property.
        }

        // 7. If requiredFields is partial and any is false, then
        if required_fields.is_none() && !any {
            // a. Throw a TypeError exception.
            return Err(JsNativeError::range()
                .with_message("requiredFields cannot be partial when any is false")
                .into());
        }

        // 8. Return result.
        Ok(result)
    }

    pub(crate) fn resolve_month(&mut self) -> JsResult<()> {
        if self.month_code.is_none() {
            if self.month.is_some() {
                return Ok(());
            }

            return Err(JsNativeError::range()
                .with_message("month and MonthCode values cannot both be undefined.")
                .into());
        }

        let unresolved_month_code = self
            .month_code
            .as_ref()
            .expect("monthCode must exist at this point.");

        let month_code_integer = month_code_to_integer(unresolved_month_code)?;

        let new_month = match self.month {
            Some(month) if month != month_code_integer => {
                return Err(JsNativeError::range()
                    .with_message("month and monthCode cannot be resolved.")
                    .into())
            }
            _ => month_code_integer,
        };

        self.month = Some(new_month);

        Ok(())
    }
}

fn month_code_to_integer(mc: &JsString) -> JsResult<i32> {
    match mc.to_std_string_escaped().as_str() {
        "M01" => Ok(1),
        "M02" => Ok(2),
        "M03" => Ok(3),
        "M04" => Ok(4),
        "M05" => Ok(5),
        "M06" => Ok(6),
        "M07" => Ok(7),
        "M08" => Ok(8),
        "M09" => Ok(9),
        "M10" => Ok(10),
        "M11" => Ok(11),
        "M12" => Ok(12),
        "M13" => Ok(13),
        _ => Err(JsNativeError::range()
            .with_message("monthCode is not within the valid values.")
            .into()),
    }
}
