//! A Rust native implementation of the `fields` object used in `Temporal`.

use crate::{
    js_string, property::PropertyKey, value::PreferredType, Context, JsNativeError, JsObject,
    JsResult, JsString, JsValue,
};

use super::options::ArithmeticOverflow;

use bitflags::bitflags;
use rustc_hash::FxHashSet;

bitflags! {
    #[derive(Debug, PartialEq, Eq)]
    pub struct FieldMap: u16 {
        const YEAR = 0b0000_0000_0000_0001;
        const MONTH = 0b0000_0000_0000_0010;
        const MONTH_CODE = 0b0000_0000_0000_0100;
        const DAY = 0b0000_0000_0000_1000;
        const HOUR = 0b0000_0000_0001_0000;
        const MINUTE = 0b0000_0000_0010_0000;
        const SECOND = 0b0000_0000_0100_0000;
        const MILLISECOND = 0b0000_0000_1000_0000;
        const MICROSECOND = 0b0000_0001_0000_0000;
        const NANOSECOND = 0b0000_0010_0000_0000;
        const OFFSET = 0b0000_0100_0000_0000;
        const ERA = 0b0000_1000_0000_0000;
        const ERA_YEAR = 0b0001_0000_0000_0000;
        const TIME_ZONE = 0b0010_0000_0000_0000;
    }
}

/// The temporal fields are laid out in the Temporal proposal under section 13.46 `PrepareTemporalFields`
/// with conversion and defaults laid out by Table 17 (displayed below).
///
/// `TemporalFields` is meant to act as a native Rust implementation
/// of the fields.
///
///
/// ## Table 17: Temporal field requirements
///
/// |   Property   |           Conversion            |  Default   |
/// | -------------|---------------------------------|------------|
/// | "year"       |     `ToIntegerWithTruncation`     | undefined  |
/// | "month"      | `ToPositiveIntegerWithTruncation` | undefined  |
/// | "monthCode"  |   `ToPrimitiveAndRequireString`   | undefined  |
/// | "day"        | `ToPositiveIntegerWithTruncation` | undefined  |
/// | "hour"       |     `ToIntegerWithTruncation`     |    +0𝔽     |
/// | "minute"     |     `ToIntegerWithTruncation`     |    +0𝔽     |
/// | "second"     |     `ToIntegerWithTruncation`     |    +0𝔽     |
/// | "millisecond"|     `ToIntegerWithTruncation`     |    +0𝔽     |
/// | "microsecond"|     `ToIntegerWithTruncation`     |    +0𝔽     |
/// | "nanosecond" |     `ToIntegerWithTruncation`     |    +0𝔽     |
/// | "offset"     |   `ToPrimitiveAndRequireString`   | undefined  |
/// | "era"        |   `ToPrimitiveAndRequireString`   | undefined  |
/// | "eraYear"    |     `ToIntegerWithTruncation`     | undefined  |
/// | "timeZone"   |                                 | undefined  |
///
#[derive(Debug)]
pub(crate) struct TemporalFields {
    bit_map: FieldMap,
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
    era_year: Option<i32>,       // TODO: switch to icu compatible value.
    time_zone: Option<JsString>, // TODO: figure out the identifier for TimeZone.
}

impl Default for TemporalFields {
    fn default() -> Self {
        Self {
            bit_map: FieldMap::empty(),
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
    pub(crate) const fn year(&self) -> Option<i32> {
        self.year
    }

    pub(crate) const fn month(&self) -> Option<i32> {
        self.month
    }

    pub(crate) const fn day(&self) -> Option<i32> {
        self.day
    }
}

impl TemporalFields {
    #[inline]
    fn set_field_value(
        &mut self,
        field: &str,
        value: &JsValue,
        context: &mut Context<'_>,
    ) -> JsResult<()> {
        match field {
            "year" => self.set_year(value, context)?,
            "month" => self.set_month(value, context)?,
            "monthCode" => self.set_month_code(value, context)?,
            "day" => self.set_day(value, context)?,
            "hour" => self.set_hour(value, context)?,
            "minute" => self.set_minute(value, context)?,
            "second" => self.set_second(value, context)?,
            "millisecond" => self.set_milli(value, context)?,
            "microsecond" => self.set_micro(value, context)?,
            "nanosecond" => self.set_nano(value, context)?,
            "offset" => self.set_offset(value, context)?,
            "era" => self.set_era(value, context)?,
            "eraYear" => self.set_era_year(value, context)?,
            "timeZone" => self.set_time_zone(value),
            _ => unreachable!(),
        }

        Ok(())
    }

    #[inline]
    fn set_year(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let y = super::to_integer_with_truncation(value, context)?;
        self.year = Some(y);
        self.bit_map.set(FieldMap::YEAR, true);
        Ok(())
    }

    #[inline]
    fn set_month(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let mo = super::to_positive_integer_with_trunc(value, context)?;
        self.year = Some(mo);
        self.bit_map.set(FieldMap::MONTH, true);
        Ok(())
    }

    #[inline]
    fn set_month_code(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let mc = value.to_primitive(context, PreferredType::String)?;
        if let Some(string) = mc.as_string() {
            self.month_code = Some(string.clone());
        } else {
            return Err(JsNativeError::typ()
                .with_message("ToPrimativeAndRequireString must be of type String.")
                .into());
        }

        self.bit_map.set(FieldMap::MONTH_CODE, true);

        Ok(())
    }

    #[inline]
    fn set_day(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let d = super::to_positive_integer_with_trunc(value, context)?;
        self.day = Some(d);
        self.bit_map.set(FieldMap::DAY, true);
        Ok(())
    }

    #[inline]
    fn set_hour(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let h = super::to_integer_with_truncation(value, context)?;
        self.hour = h;
        self.bit_map.set(FieldMap::HOUR, true);
        Ok(())
    }

    #[inline]
    fn set_minute(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let m = super::to_integer_with_truncation(value, context)?;
        self.minute = m;
        self.bit_map.set(FieldMap::MINUTE, true);
        Ok(())
    }

    #[inline]
    fn set_second(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let sec = super::to_integer_with_truncation(value, context)?;
        self.second = sec;
        self.bit_map.set(FieldMap::SECOND, true);
        Ok(())
    }

    #[inline]
    fn set_milli(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let milli = super::to_integer_with_truncation(value, context)?;
        self.millisecond = milli;
        self.bit_map.set(FieldMap::MILLISECOND, true);
        Ok(())
    }

    #[inline]
    fn set_micro(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let micro = super::to_integer_with_truncation(value, context)?;
        self.microsecond = micro;
        self.bit_map.set(FieldMap::MICROSECOND, true);
        Ok(())
    }

    #[inline]
    fn set_nano(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let nano = super::to_integer_with_truncation(value, context)?;
        self.nanosecond = nano;
        self.bit_map.set(FieldMap::NANOSECOND, true);
        Ok(())
    }

    #[inline]
    fn set_offset(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let mc = value.to_primitive(context, PreferredType::String)?;
        if let Some(string) = mc.as_string() {
            self.offset = Some(string.clone());
        } else {
            return Err(JsNativeError::typ()
                .with_message("ToPrimativeAndRequireString must be of type String.")
                .into());
        }
        self.bit_map.set(FieldMap::OFFSET, true);

        Ok(())
    }

    #[inline]
    fn set_era(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let mc = value.to_primitive(context, PreferredType::String)?;
        if let Some(string) = mc.as_string() {
            self.era = Some(string.clone());
        } else {
            return Err(JsNativeError::typ()
                .with_message("ToPrimativeAndRequireString must be of type String.")
                .into());
        }
        self.bit_map.set(FieldMap::ERA, true);

        Ok(())
    }

    #[inline]
    fn set_era_year(&mut self, value: &JsValue, context: &mut Context<'_>) -> JsResult<()> {
        let ey = super::to_integer_with_truncation(value, context)?;
        self.era_year = Some(ey);
        self.bit_map.set(FieldMap::ERA_YEAR, true);
        Ok(())
    }

    #[inline]
    fn set_time_zone(&mut self, value: &JsValue) {
        let tz = value.as_string().cloned();
        self.time_zone = tz;
        self.bit_map.set(FieldMap::TIME_ZONE, true);
    }
}

impl TemporalFields {
    // TODO: Shift to JsString or utf16 over String.
    /// A method for creating a Native representation for `TemporalFields` from
    /// a `JsObject`.
    ///
    /// This is the equivalant to Abstract Operation 13.46 `PrepareTemporalFields`
    pub(crate) fn from_js_object(
        fields: &JsObject,
        field_names: &mut Vec<String>,
        required_fields: &mut Vec<String>, // None when Partial
        extended_fields: Option<Vec<(String, bool)>>,
        partial: bool,
        dup_behaviour: Option<JsString>,
        context: &mut Context<'_>,
    ) -> JsResult<Self> {
        // 1. If duplicateBehaviour is not present, set duplicateBehaviour to throw.
        let dup_option = dup_behaviour.unwrap_or_else(|| js_string!("throw"));

        // 2. Let result be OrdinaryObjectCreate(null).
        let mut result = Self::default();

        // 3. Let any be false.
        let mut any = false;
        // 4. If extraFieldDescriptors is present, then
        if let Some(extra_fields) = extended_fields {
            for (field_name, required) in extra_fields {
                // a. For each Calendar Field Descriptor Record desc of extraFieldDescriptors, do
                // i. Assert: fieldNames does not contain desc.[[Property]].
                // ii. Append desc.[[Property]] to fieldNames.
                field_names.push(field_name.clone());

                // iii. If desc.[[Required]] is true and requiredFields is a List, then
                if required && !partial {
                    // 1. Append desc.[[Property]] to requiredFields.
                    required_fields.push(field_name);
                }
            }
        }

        // 5. Let sortedFieldNames be SortStringListByCodeUnit(fieldNames).
        // 6. Let previousProperty be undefined.
        let mut dups_map = FxHashSet::default();

        // 7. For each property name property of sortedFieldNames, do
        for field in &*field_names {
            // a. If property is one of "constructor" or "__proto__", then
            if field.as_str() == "constructor" || field.as_str() == "__proto__" {
                // i. Throw a RangeError exception.
                return Err(JsNativeError::range()
                    .with_message("constructor or proto is out of field range.")
                    .into());
            }

            let new_value = dups_map.insert(field);

            // b. If property is not equal to previousProperty, then
            if new_value {
                // i. Let value be ? Get(fields, property).
                let value =
                    fields.get(PropertyKey::from(JsString::from(field.clone())), context)?;
                // ii. If value is not undefined, then
                if !value.is_undefined() {
                    // 1. Set any to true.
                    any = true;

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
                    result.set_field_value(field, &value, context)?;
                // iii. Else if requiredFields is a List, then
                } else if !partial {
                    // 1. If requiredFields contains property, then
                    if required_fields.contains(field) {
                        // a. Throw a TypeError exception.
                        return Err(JsNativeError::typ()
                            .with_message("A required TemporalField was not provided.")
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

        // 8. If requiredFields is partial and any is false, then
        if partial && !any {
            // a. Throw a TypeError exception.
            return Err(JsNativeError::range()
                .with_message("requiredFields cannot be partial when any is false")
                .into());
        }

        // 9. Return result.
        Ok(result)
    }

    /// Convert a `TemporalFields` struct into a `JsObject`.
    pub(crate) fn as_object(&self, context: &mut Context<'_>) -> JsResult<JsObject> {
        let obj = JsObject::with_null_proto();

        for bit in self.bit_map.iter() {
            match bit {
                FieldMap::YEAR => {
                    obj.create_data_property_or_throw(
                        js_string!("year"),
                        self.year.map_or(JsValue::undefined(), JsValue::from),
                        context,
                    )?;
                }
                FieldMap::MONTH => {
                    obj.create_data_property_or_throw(
                        js_string!("month"),
                        self.month.map_or(JsValue::undefined(), JsValue::from),
                        context,
                    )?;
                }
                FieldMap::MONTH_CODE => {
                    obj.create_data_property_or_throw(
                        js_string!("monthCode"),
                        self.month_code
                            .as_ref()
                            .map_or(JsValue::undefined(), |f| f.clone().into()),
                        context,
                    )?;
                }
                FieldMap::DAY => {
                    obj.create_data_property(
                        js_string!("day"),
                        self.day().map_or(JsValue::undefined(), JsValue::from),
                        context,
                    )?;
                }
                FieldMap::HOUR => {
                    obj.create_data_property(js_string!("hour"), self.hour, context)?;
                }
                FieldMap::MINUTE => {
                    obj.create_data_property(js_string!("minute"), self.minute, context)?;
                }
                FieldMap::SECOND => {
                    obj.create_data_property_or_throw(js_string!("second"), self.second, context)?;
                }
                FieldMap::MILLISECOND => {
                    obj.create_data_property_or_throw(
                        js_string!("millisecond"),
                        self.millisecond,
                        context,
                    )?;
                }
                FieldMap::MICROSECOND => {
                    obj.create_data_property_or_throw(
                        js_string!("microsecond"),
                        self.microsecond,
                        context,
                    )?;
                }
                FieldMap::NANOSECOND => {
                    obj.create_data_property_or_throw(
                        js_string!("nanosecond"),
                        self.nanosecond,
                        context,
                    )?;
                }
                FieldMap::OFFSET => {
                    obj.create_data_property_or_throw(
                        js_string!("offset"),
                        self.offset
                            .as_ref()
                            .map_or(JsValue::undefined(), |s| s.clone().into()),
                        context,
                    )?;
                }
                FieldMap::ERA => {
                    obj.create_data_property_or_throw(
                        js_string!("era"),
                        self.era
                            .as_ref()
                            .map_or(JsValue::undefined(), |s| s.clone().into()),
                        context,
                    )?;
                }
                FieldMap::ERA_YEAR => {
                    obj.create_data_property_or_throw(
                        js_string!("eraYear"),
                        self.era_year.map_or(JsValue::undefined(), JsValue::from),
                        context,
                    )?;
                }
                FieldMap::TIME_ZONE => {
                    obj.create_data_property_or_throw(
                        js_string!("timeZone"),
                        self.time_zone
                            .as_ref()
                            .map_or(JsValue::undefined(), |s| s.clone().into()),
                        context,
                    )?;
                }
                _ => unreachable!(),
            }
        }

        Ok(obj)
    }

    // Note placeholder until overflow is implemented on `ICU4x`'s Date<Iso>.
    /// A function to regulate the current `TemporalFields` according to the overflow value
    pub(crate) fn regulate(&mut self, overflow: ArithmeticOverflow) -> JsResult<()> {
        if let (Some(year), Some(month), Some(day)) = (self.year(), self.month(), self.day()) {
            match overflow {
                ArithmeticOverflow::Constrain => {
                    let m = month.clamp(1, 12);
                    let days_in_month = super::calendar::utils::iso_days_in_month(year, month);
                    let d = day.clamp(1, days_in_month);

                    self.month = Some(m);
                    self.day = Some(d);
                }
                ArithmeticOverflow::Reject => {
                    return Err(JsNativeError::range()
                        .with_message("TemporalFields is out of a valid range.")
                        .into())
                }
            }
        }
        Ok(())
    }

    pub(crate) fn regulate_year_month(&mut self, overflow: ArithmeticOverflow) {
        match self.month {
            Some(month) if overflow == ArithmeticOverflow::Constrain => {
                let m = month.clamp(1, 12);
                self.month = Some(m);
            }
            _ => {}
        }
    }

    /// Resolve the month and monthCode on this `TemporalFields`.
    pub(crate) fn iso_resolve_month(&mut self) -> JsResult<()> {
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
