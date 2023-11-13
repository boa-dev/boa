//! A Rust native implementation of the `fields` object used in `Temporal`.

use std::str::FromStr;

use crate::{
    js_string, property::PropertyKey, value::PreferredType, Context, JsNativeError, JsObject,
    JsResult, JsString, JsValue,
};

use rustc_hash::FxHashSet;

use boa_temporal::fields::{FieldConversion, FieldValue, TemporalFields};

use super::{to_integer_with_truncation, to_positive_integer_with_trunc};

// TODO: Move extended and required fields into the temporal library?
/// `PrepareTemporalFeilds`
pub(crate) fn prepare_temporal_fields(
    fields: &JsObject,
    field_names: &mut Vec<JsString>,
    required_fields: &mut Vec<JsString>,
    extended_fields: Option<Vec<(String, bool)>>,
    partial: bool,
    dup_behaviour: Option<JsString>,
    context: &mut Context,
) -> JsResult<TemporalFields> {
    // 1. If duplicateBehaviour is not present, set duplicateBehaviour to throw.
    let dup_option = dup_behaviour.unwrap_or_else(|| js_string!("throw"));

    // 2. Let result be OrdinaryObjectCreate(null).
    let mut result = TemporalFields::default();

    // 3. Let any be false.
    let mut any = false;
    // 4. If extraFieldDescriptors is present, then
    if let Some(extra_fields) = extended_fields {
        for (field_name, required) in extra_fields {
            // a. For each Calendar Field Descriptor Record desc of extraFieldDescriptors, do
            // i. Assert: fieldNames does not contain desc.[[Property]].
            // ii. Append desc.[[Property]] to fieldNames.
            field_names.push(JsString::from(field_name.clone()));

            // iii. If desc.[[Required]] is true and requiredFields is a List, then
            if required && !partial {
                // 1. Append desc.[[Property]] to requiredFields.
                required_fields.push(JsString::from(field_name));
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
            return Err(JsNativeError::range()
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

                // TODO: Conversion from TemporalError -> JsError
                let conversion = FieldConversion::from_str(field.to_std_string_escaped().as_str())
                    .map_err(|_| JsNativeError::range().with_message("wrong field value"))?;
                // b. If Conversion is ToIntegerWithTruncation, then
                let converted_value = match conversion {
                    FieldConversion::ToIntegerWithTruncation => {
                        // i. Set value to ? ToIntegerWithTruncation(value).
                        let v = to_integer_with_truncation(&value, context)?;
                        // ii. Set value to 𝔽(value).
                        FieldValue::Integer(v)
                    }
                    // c. Else if Conversion is ToPositiveIntegerWithTruncation, then
                    FieldConversion::ToPositiveIntegerWithTruncation => {
                        // i. Set value to ? ToPositiveIntegerWithTruncation(value).
                        let v = to_positive_integer_with_trunc(&value, context)?;
                        // ii. Set value to 𝔽(value).
                        FieldValue::Integer(v)
                    }
                    // d. Else,
                    // i. Assert: Conversion is ToPrimitiveAndRequireString.
                    FieldConversion::ToPrimativeAndRequireString => {
                        // ii. NOTE: Non-primitive values are supported here for consistency with other fields, but such values must coerce to Strings.
                        // iii. Set value to ? ToPrimitive(value, string).
                        let primitive = value.to_primitive(context, PreferredType::String)?;
                        // iv. If value is not a String, throw a TypeError exception.
                        FieldValue::String(primitive.to_string(context)?.to_std_string_escaped())
                    }
                    FieldConversion::None => {
                        unreachable!("todo need to implement conversion handling for tz.")
                    }
                };

                // 3. Perform ! CreateDataPropertyOrThrow(result, property, value).
                result
                    .set_field_value(&field.to_std_string_escaped(), &converted_value)
                    .expect("FieldConversion enforces the appropriate type");
            // iii. Else if requiredFields is a List, then
            } else if !partial {
                // 1. If requiredFields contains property, then
                if required_fields.contains(field) {
                    // a. Throw a TypeError exception.
                    return Err(JsNativeError::typ()
                        .with_message("A required TemporalField was not provided.")
                        .into());
                }

                // NOTE: flag that the value is active and the default should be used.
                // 2. If property is in the Property column of Table 17, then
                // a. Set value to the corresponding Default value of the same row.
                // 3. Perform ! CreateDataPropertyOrThrow(result, property, value).
                result.require_field(&field.to_std_string_escaped());
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

impl JsObject {
    pub(crate) fn from_temporal_fields(
        fields: &TemporalFields,
        context: &mut Context,
    ) -> JsResult<Self> {
        let obj = JsObject::with_null_proto();

        for (key, value) in fields.active_kvs() {
            let js_value = match value {
                FieldValue::Undefined => JsValue::undefined(),
                FieldValue::Integer(x) => JsValue::Integer(x),
                FieldValue::String(s) => JsValue::String(s.into()),
            };

            obj.create_data_property_or_throw(JsString::from(key), js_value, context)?;
        }

        Ok(obj)
    }
}
