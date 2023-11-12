//! `TemporalFields` native Rust representation.

use std::str::FromStr;

use crate::{error::TemporalError, TemporalResult};

use bitflags::bitflags;
// use rustc_hash::FxHashSet;
use tinystr::{TinyStr16, TinyStr4};

bitflags! {
    /// FieldMap maps the currently active fields on the `TemporalField`
    #[derive(Debug, PartialEq, Eq)]
    pub struct FieldMap: u16 {
        /// Represents an active `year` field
        const YEAR = 0b0000_0000_0000_0001;
        /// Represents an active `month` field
        const MONTH = 0b0000_0000_0000_0010;
        /// Represents an active `monthCode` field
        const MONTH_CODE = 0b0000_0000_0000_0100;
        /// Represents an active `day` field
        const DAY = 0b0000_0000_0000_1000;
        /// Represents an active `hour` field
        const HOUR = 0b0000_0000_0001_0000;
        /// Represents an active `minute` field
        const MINUTE = 0b0000_0000_0010_0000;
        /// Represents an active `second` field
        const SECOND = 0b0000_0000_0100_0000;
        /// Represents an active `millisecond` field
        const MILLISECOND = 0b0000_0000_1000_0000;
        /// Represents an active `microsecond` field
        const MICROSECOND = 0b0000_0001_0000_0000;
        /// Represents an active `nanosecond` field
        const NANOSECOND = 0b0000_0010_0000_0000;
        /// Represents an active `offset` field
        const OFFSET = 0b0000_0100_0000_0000;
        /// Represents an active `era` field
        const ERA = 0b0000_1000_0000_0000;
        /// Represents an active `eraYear` field
        const ERA_YEAR = 0b0001_0000_0000_0000;
        /// Represents an active `timeZone` field
        const TIME_ZONE = 0b0010_0000_0000_0000;
    }
}

/// The post conversion field value.
#[derive(Debug)]
pub enum FieldValue {
    /// Designates the values as an integer.
    Integer(i32),
    /// Designates the value as a string.
    String(String),
    /// Designates that the value is undefined.
    Undefined,
}

/// The Conversion type of a field.
#[derive(Debug, Clone, Copy)]
pub enum FieldConversion {
    /// Designates the Conversion type is `ToIntegerWithTruncation`
    ToIntegerWithTruncation,
    /// Designates the Conversion type is `ToPositiveIntegerWithTruncation`
    ToPositiveIntegerWithTruncation,
    /// Designates the Conversion type is `ToPrimitiveRequireString`
    ToPrimativeAndRequireString,
    /// Designates the Conversion type is nothing
    None,
}

impl FromStr for FieldConversion {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "year" | "hour" | "minute" | "second" | "millisecond" | "microsecond"
            | "nanosecond" => Ok(Self::ToIntegerWithTruncation),
            "month" | "day" => Ok(Self::ToPositiveIntegerWithTruncation),
            "monthCode" | "offset" | "eraYear" => Ok(Self::ToPrimativeAndRequireString),
            _ => Err(TemporalError::range()
                .with_message(format!("{s} is not a valid TemporalField Property"))),
        }
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
/// |   Property   |           Conversion              |  Default   |
/// | -------------|-----------------------------------|------------|
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
/// | "timeZone"   |              `None`               | undefined  |
#[derive(Debug)]
pub struct TemporalFields {
    bit_map: FieldMap,
    year: Option<i32>,
    month: Option<i32>,
    month_code: Option<TinyStr4>, // TODO: Switch to icu compatible value.
    day: Option<i32>,
    hour: i32,
    minute: i32,
    second: i32,
    millisecond: i32,
    microsecond: i32,
    nanosecond: i32,
    offset: Option<String>,    // TODO: Switch to tinystr?
    era: Option<TinyStr16>,    // TODO: switch to icu compatible value.
    era_year: Option<i32>,     // TODO: switch to icu compatible value.
    time_zone: Option<String>, // TODO: figure out the identifier for TimeZone.
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

// TODO: Update the below.
impl TemporalFields {
    /// Flags a field as being required.
    #[inline]
    pub fn require_field(&mut self, field: String) {
        match field.as_str() {
            "year" => self.bit_map.set(FieldMap::YEAR, true),
            "month" => self.bit_map.set(FieldMap::MONTH, true),
            "monthCode" => self.bit_map.set(FieldMap::MONTH_CODE, true),
            "day" => self.bit_map.set(FieldMap::DAY, true),
            "hour" => self.bit_map.set(FieldMap::HOUR, true),
            "minute" => self.bit_map.set(FieldMap::MINUTE, true),
            "second" => self.bit_map.set(FieldMap::SECOND, true),
            "millisecond" => self.bit_map.set(FieldMap::MILLISECOND, true),
            "microsecond" => self.bit_map.set(FieldMap::MICROSECOND, true),
            "nanosecond" => self.bit_map.set(FieldMap::NANOSECOND, true),
            "offset" => self.bit_map.set(FieldMap::OFFSET, true),
            "era" => self.bit_map.set(FieldMap::ERA, true),
            "eraYear" => self.bit_map.set(FieldMap::ERA_YEAR, true),
            "timeZone" => self.bit_map.set(FieldMap::TIME_ZONE, true),
            _ => {}
        }
    }

    #[inline]
    /// A generic field setter for `TemporalFields`
    ///
    /// This method will not run any `JsValue` conversion. `FieldValue` is
    /// expected to contain a preconverted value.
    pub fn set_field_value(
        &mut self,
        field: String, // TODO: Switch to an options Enum.
        value: FieldValue,
    ) -> TemporalResult<()> {
        match field.as_str() {
            "year" => self.set_year(value)?,
            "month" => self.set_month(value)?,
            "monthCode" => self.set_month_code(value)?,
            "day" => self.set_day(value)?,
            "hour" => self.set_hour(value)?,
            "minute" => self.set_minute(value)?,
            "second" => self.set_second(value)?,
            "millisecond" => self.set_milli(value)?,
            "microsecond" => self.set_micro(value)?,
            "nanosecond" => self.set_nano(value)?,
            "offset" => self.set_offset(value)?,
            "era" => self.set_era(value)?,
            "eraYear" => self.set_era_year(value)?,
            "timeZone" => self.set_time_zone(value)?,
            _ => unreachable!(),
        }

        Ok(())
    }

    #[inline]
    fn set_year(&mut self, value: FieldValue) -> TemporalResult<()> {
        let FieldValue::Integer(y) = value else {
            return Err(TemporalError::r#type().with_message("Year must be an integer."));
        };
        self.year = Some(y);
        self.bit_map.set(FieldMap::YEAR, true);
        Ok(())
    }

    #[inline]
    fn set_month(&mut self, value: FieldValue) -> TemporalResult<()> {
        let FieldValue::Integer(mo) = value else {
            return Err(TemporalError::r#type().with_message("Month must be an integer."));
        };
        self.year = Some(mo);
        self.bit_map.set(FieldMap::MONTH, true);
        Ok(())
    }

    #[inline]
    fn set_month_code(&mut self, value: FieldValue) -> TemporalResult<()> {
        let FieldValue::String(mc) = value else {
            return Err(TemporalError::r#type().with_message("monthCode must be string."));
        };
        self.month_code =
            Some(TinyStr4::from_bytes(mc.as_bytes()).expect("monthCode must be less than 4 chars"));
        self.bit_map.set(FieldMap::MONTH_CODE, true);
        Ok(())
    }

    #[inline]
    fn set_day(&mut self, value: FieldValue) -> TemporalResult<()> {
        let FieldValue::Integer(d) = value else {
            return Err(TemporalError::r#type().with_message("day must be an integer."));
        };
        self.day = Some(d);
        self.bit_map.set(FieldMap::DAY, true);
        Ok(())
    }

    #[inline]
    fn set_hour(&mut self, value: FieldValue) -> TemporalResult<()> {
        let FieldValue::Integer(h) = value else {
            return Err(TemporalError::r#type().with_message("hour must be an integer."));
        };
        self.hour = h;
        self.bit_map.set(FieldMap::HOUR, true);
        Ok(())
    }

    #[inline]
    fn set_minute(&mut self, value: FieldValue) -> TemporalResult<()> {
        let FieldValue::Integer(min) = value else {
            return Err(TemporalError::r#type().with_message("minute must be an integer."));
        };
        self.minute = min;
        self.bit_map.set(FieldMap::MINUTE, true);
        Ok(())
    }

    #[inline]
    fn set_second(&mut self, value: FieldValue) -> TemporalResult<()> {
        let FieldValue::Integer(sec) = value else {
            return Err(TemporalError::r#type().with_message("Second must be an integer."));
        };
        self.second = sec;
        self.bit_map.set(FieldMap::SECOND, true);
        Ok(())
    }

    #[inline]
    fn set_milli(&mut self, value: FieldValue) -> TemporalResult<()> {
        let FieldValue::Integer(milli) = value else {
            return Err(TemporalError::r#type().with_message("Second must be an integer."));
        };
        self.millisecond = milli;
        self.bit_map.set(FieldMap::MILLISECOND, true);
        Ok(())
    }

    #[inline]
    fn set_micro(&mut self, value: FieldValue) -> TemporalResult<()> {
        let FieldValue::Integer(micro) = value else {
            return Err(TemporalError::r#type().with_message("microsecond must be an integer."));
        };
        self.microsecond = micro;
        self.bit_map.set(FieldMap::MICROSECOND, true);
        Ok(())
    }

    #[inline]
    fn set_nano(&mut self, value: FieldValue) -> TemporalResult<()> {
        let FieldValue::Integer(nano) = value else {
            return Err(TemporalError::r#type().with_message("nanosecond must be an integer."));
        };
        self.nanosecond = nano;
        self.bit_map.set(FieldMap::NANOSECOND, true);
        Ok(())
    }

    #[inline]
    fn set_offset(&mut self, value: FieldValue) -> TemporalResult<()> {
        let FieldValue::String(offset) = value else {
            return Err(TemporalError::r#type().with_message("offset must be string."));
        };
        self.offset = Some(offset);
        self.bit_map.set(FieldMap::OFFSET, true);

        Ok(())
    }

    #[inline]
    fn set_era(&mut self, value: FieldValue) -> TemporalResult<()> {
        let FieldValue::String(era) = value else {
            return Err(TemporalError::r#type().with_message("era must be string."));
        };
        self.era =
            Some(TinyStr16::from_bytes(era.as_bytes()).expect("era should not exceed 16 bytes."));
        self.bit_map.set(FieldMap::ERA, true);

        Ok(())
    }

    #[inline]
    fn set_era_year(&mut self, value: FieldValue) -> TemporalResult<()> {
        let FieldValue::Integer(era_year) = value else {
            return Err(TemporalError::r#type().with_message("eraYear must be an integer."));
        };
        self.era_year = Some(era_year);
        self.bit_map.set(FieldMap::ERA_YEAR, true);
        Ok(())
    }

    #[inline]
    fn set_time_zone(&mut self, value: FieldValue) -> TemporalResult<()> {
        let FieldValue::String(tz) = value else {
            return Err(TemporalError::r#type().with_message("tz must be string."));
        };
        self.time_zone = Some(tz);
        self.bit_map.set(FieldMap::TIME_ZONE, true);
        Ok(())
    }
}

// TODO: optimize into iter.
impl TemporalFields {
    /// Returns a vector filled with the key-value pairs marked as active.
    pub fn active_kvs(&self) -> Vec<(String, FieldValue)> {
        let mut result = Vec::default();

        for field in self.bit_map.iter() {
            match field {
                FieldMap::YEAR => result.push((
                    "year".to_owned(),
                    self.year.map_or(FieldValue::Undefined, FieldValue::Integer),
                )),
                FieldMap::MONTH => result.push((
                    "month".to_owned(),
                    self.month
                        .map_or(FieldValue::Undefined, FieldValue::Integer),
                )),
                FieldMap::MONTH_CODE => result.push((
                    "monthCode".to_owned(),
                    self.month_code
                        .map_or(FieldValue::Undefined, |s| FieldValue::String(s.to_string())),
                )),
                FieldMap::DAY => result.push((
                    "day".to_owned(),
                    self.day.map_or(FieldValue::Undefined, FieldValue::Integer),
                )),
                FieldMap::HOUR => result.push(("hour".to_owned(), FieldValue::Integer(self.hour))),
                FieldMap::MINUTE => {
                    result.push(("minute".to_owned(), FieldValue::Integer(self.minute)));
                }
                FieldMap::SECOND => {
                    result.push(("second".to_owned(), FieldValue::Integer(self.second)));
                }
                FieldMap::MILLISECOND => result.push((
                    "millisecond".to_owned(),
                    FieldValue::Integer(self.millisecond),
                )),
                FieldMap::MICROSECOND => result.push((
                    "microsecond".to_owned(),
                    FieldValue::Integer(self.microsecond),
                )),
                FieldMap::NANOSECOND => result.push((
                    "nanosecond".to_owned(),
                    FieldValue::Integer(self.nanosecond),
                )),
                FieldMap::OFFSET => result.push((
                    "offset".to_owned(),
                    self.offset
                        .clone()
                        .map_or(FieldValue::Undefined, FieldValue::String),
                )),
                FieldMap::ERA => result.push((
                    "era".to_owned(),
                    self.era
                        .map_or(FieldValue::Undefined, |s| FieldValue::String(s.to_string())),
                )),
                FieldMap::ERA_YEAR => result.push((
                    "eraYear".to_owned(),
                    self.era_year
                        .map_or(FieldValue::Undefined, FieldValue::Integer),
                )),
                FieldMap::TIME_ZONE => result.push((
                    "timeZone".to_owned(),
                    self.time_zone
                        .clone()
                        .map_or(FieldValue::Undefined, FieldValue::String),
                )),
                _ => {}
            }
        }

        result
    }

    /// Resolve `TemporalFields` month and monthCode fields.
    pub(crate) fn iso_resolve_month(&mut self) -> TemporalResult<()> {
        if self.month_code.is_none() {
            if self.month.is_some() {
                return Ok(());
            }

            return Err(TemporalError::range()
                .with_message("month and MonthCode values cannot both be undefined."));
        }

        let unresolved_month_code = self
            .month_code
            .as_ref()
            .expect("monthCode must exist at this point.");

        let month_code_integer = month_code_to_integer(unresolved_month_code)?;

        let new_month = match self.month {
            Some(month) if month != month_code_integer => {
                return Err(
                    TemporalError::range().with_message("month and monthCode cannot be resolved.")
                )
            }
            _ => month_code_integer,
        };

        self.month = Some(new_month);

        Ok(())
    }
}

fn month_code_to_integer(mc: &TinyStr4) -> TemporalResult<i32> {
    match mc.as_str() {
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
        _ => Err(TemporalError::range().with_message("monthCode is not within the valid values.")),
    }
}

/*
impl TemporalFields {
    // TODO: Shift to JsString or utf16 over String.
    /// A method for creating a Native representation for `TemporalFields` from
    /// a `JsObject`.
    ///
    /// This is the equivalant to Abstract Operation 13.46 `PrepareTemporalFields`
    pub(crate) fn from_js_object(
        fields: &JsObject,
        field_names: &mut Vec<JsString>,
        required_fields: &mut Vec<JsString>, // None when Partial
        extended_fields: Option<Vec<(JsString, bool)>>,
        partial: bool,
        dup_behaviour: Option<JsString>,
        context: &mut Context<'_>,
    ) -> TemporalResult<Self> {
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
            if field.to_std_string_escaped().as_str() == "constructor"
                || field.to_std_string_escaped().as_str() == "__proto__"
            {
                // i. Throw a RangeError exception.
                return Err(TemporalError::range()
                    .with_message("constructor or proto is out of field range.")
                    .into());
            }

            let new_value = dups_map.insert(field);

            // b. If property is not equal to previousProperty, then
            if new_value {
                // i. Let value be ? Get(fields, property).
                let value = fields.get(PropertyKey::from(field.clone()), context)?;
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
                        return Err(TemporalError::typ()
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
                return Err(TemporalError::range()
                    .with_message("Cannot have a duplicate field")
                    .into());
            }
            // d. Set previousProperty to property.
        }

        // 8. If requiredFields is partial and any is false, then
        if partial && !any {
            // a. Throw a TypeError exception.
            return Err(TemporalError::range()
                .with_message("requiredFields cannot be partial when any is false")
                .into());
        }

        // 9. Return result.
        Ok(result)
    }

    /// Convert a `TemporalFields` struct into a `JsObject`.
    pub(crate) fn as_object(&self, context: &mut Context<'_>) -> TemporalResult<JsObject> {
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
    pub(crate) fn regulate(&mut self, overflow: ArithmeticOverflow) -> TemporalResult<()> {
        if let (Some(year), Some(month), Some(day)) = (self.year(), self.month(), self.day()) {
            match overflow {
                ArithmeticOverflow::Constrain => {
                    let m = month.clamp(1, 12);
                    let days_in_month = crate::utils::iso_days_in_month(year, month);
                    let d = day.clamp(1, days_in_month);

                    self.month = Some(m);
                    self.day = Some(d);
                }
                ArithmeticOverflow::Reject => {
                    return Err(TemporalError::range()
                        .with_message("TemporalFields is out of a valid range.")
                        )
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

}



*/
