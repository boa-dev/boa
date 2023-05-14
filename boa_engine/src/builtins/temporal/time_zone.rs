#![allow(dead_code)]

use boa_ast::{temporal::OffsetSign, UtcOffset};
use boa_parser::parser::UTCOffset;

use crate::{
    builtins::{
        temporal::to_zero_padded_decimal_string, BuiltInBuilder, BuiltInConstructor, BuiltInObject,
        IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder,
        FunctionObjectBuilder, ObjectData, ObjectInitializer, CONSTRUCTOR,
    },
    property::Attribute,
    realm::Realm,
    string::utf16,
    value::{AbstractRelation, IntegerOrInfinity},
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_profiler::Profiler;

/// The `Temporal.TimeZone` object.
#[derive(Debug, Clone)]
pub struct TimeZone {
    pub(crate) initialized_temporal_time_zone: bool,
    pub(crate) identifier: String,
    pub(crate) offset_nanoseconds: Option<i64>,
}

impl BuiltInObject for TimeZone {
    const NAME: &'static str = "Temporal.TimeZone";
}

impl IntrinsicObject for TimeZone {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let get_id = BuiltInBuilder::new(realm)
            .callable(Self::get_id)
            .name("get Id")
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(
                Self::get_offset_nanoseconds_for,
                "getOffsetNanosecondsFor",
                1,
            )
            .method(Self::get_offset_string_for, "getOffsetStringFor", 1)
            .method(Self::get_plain_date_time_for, "getPlainDateTimeFor", 2)
            .method(Self::get_instant_for, "getInstantFor", 2)
            .method(Self::get_possible_instants_for, "getPossibleInstantFor", 1)
            .method(Self::get_next_transition, "getNextTransition", 1)
            .method(Self::get_previous_transition, "getPreviousTransition", 1)
            .method(Self::to_string, "toString", 0)
            .method(Self::to_string, "toJSON", 0)
            .static_property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                CONSTRUCTOR,
                realm.intrinsics().constructors().time_zone().prototype(),
                Attribute::default(),
            )
            .accessor(utf16!("id"), Some(get_id), None, Attribute::default())
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for TimeZone {
    const LENGTH: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::time_zone;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, then
        // 1a. Throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("newTarget cannot be undefined for Temporal.TimeZone constructor")
                .into());
        };

        // 2. Set identifier to ? ToString(identifier).
        let identifier = args.get_or_undefined(0);
        if identifier.is_undefined() {
            return Err(JsNativeError::range()
                .with_message("Temporal.TimeZone must be called with a valid initializer")
                .into());
        }

        // 3. If IsTimeZoneOffsetString(identifier) is false, then
        //    a. If IsAvailableTimeZoneName(identifier) is false, then
        //        i. Throw a RangeError exception.
        //    b. Set identifier to ! CanonicalizeTimeZoneName(identifier).
        // 4. Return ? CreateTemporalTimeZone(identifier, NewTarget).
        create_temporal_time_zone(
            identifier.to_string(context)?.to_std_string_escaped(),
            Some(new_target.clone()),
            context,
        )
    }
}

impl TimeZone {
    // NOTE: id, toJSON, toString currently share the exact same implementation -> Consolidate into one function and define multiple accesors?
    pub(crate) fn get_id(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value must be a Temporal.TimeZone")
        })?;
        let o = o.borrow();
        let tz = o.as_time_zone().ok_or_else(|| {
            JsNativeError::typ().with_message("this value must be a Temporal.TimeZone")
        })?;
        Ok(tz.identifier.clone().into())
    }

    pub(crate) fn get_offset_nanoseconds_for(
        this: &JsValue,
        args: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let timeZone be the this value.
        // 2. Perform ? RequireInternalSlot(timeZone, [[InitializedTemporalTimeZone]]).
        let _tz = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ().with_message("this value must be a Temporal.TimeZone")
            })?
            .borrow()
            .as_time_zone()
            .ok_or_else(|| {
                JsNativeError::typ().with_message("this value must be a Temporal.TimeZone")
            })?;
        // 3. Set instant to ? ToTemporalInstant(instant).
        let _i = args.get_or_undefined(0);
        // TODO: to_temporal_instant is abstract operation for Temporal.Instant objects.
        // let instant = to_temporal_instant(i)?;
        todo!()

        // 4. If timeZone.[[OffsetNanoseconds]] is not undefined, return 𝔽(timeZone.[[OffsetNanoseconds]]).
        // 5. Return 𝔽(GetNamedTimeZoneOffsetNanoseconds(timeZone.[[Identifier]], instant.[[Nanoseconds]])).
    }

    pub(crate) fn get_offset_string_for(
        this: &JsValue,
        args: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let timeZone be the this value.
        // 2. Perform ? RequireInternalSlot(timeZone, [[InitializedTemporalTimeZone]]).
        let _tz = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ().with_message("this value must be a Temporal.TimeZone")
            })?
            .borrow()
            .as_time_zone()
            .ok_or_else(|| {
                JsNativeError::typ().with_message("this value must be a Temporal.TimeZone")
            })?;
        // 3. Set instant to ? ToTemporalInstant(instant).
        let _i = args.get_or_undefined(0);
        // TODO: to_temporal_instant is abstract operation for Temporal.Instant objects.
        // let instant = to_temporal_instant(i)?;
        todo!()

        // 4. Return ? GetOffsetStringFor(timeZone, instant).
    }

    pub(crate) fn get_plain_date_time_for(
        _: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        todo!()
    }

    pub(crate) fn get_instant_for(
        _: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        todo!()
    }

    pub(crate) fn get_possible_instants_for(
        _: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        todo!()
    }

    pub(crate) fn get_next_transition(
        _: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        todo!()
    }

    pub(crate) fn get_previous_transition(
        _: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        todo!()
    }

    pub(crate) fn to_string(
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let timeZone be the this value.
        // 2. Perform ? RequireInternalSlot(timeZone, [[InitializedTemporalTimeZone]]).
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value must be a Temporal.TimeZone")
        })?;
        let o = o.borrow();
        let tz = o.as_time_zone().ok_or_else(|| {
            JsNativeError::typ().with_message("this value must be a Temporal.TimeZone")
        })?;
        // 3. Return timeZone.[[Identifier]].
        Ok(tz.identifier.clone().into())
    }
}

// -- TimeZone Abstract Operations --

/// Abstract operation `DefaultTimeZone ( )`
///
/// The abstract operation `DefaultTimeZone` takes no arguments. It returns a String value
/// representing the host environment's current time zone, which is either a valid (11.1.1) and
/// canonicalized (11.1.2) time zone name, or an offset conforming to the syntax of a
/// `TimeZoneNumericUTCOffset`.
///
/// An ECMAScript implementation that includes the ECMA-402 Internationalization API must implement
/// the `DefaultTimeZone` abstract operation as specified in the ECMA-402 specification.
///
/// More information:
///  - [ECMAScript specififcation][spec]
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-defaulttimezone
#[allow(unused)]
pub(super) fn default_time_zone(context: &mut Context<'_>) -> String {
    // The minimum implementation of DefaultTimeZone for ECMAScript implementations that do not
    // include the ECMA-402 API, supporting only the "UTC" time zone, performs the following steps
    // when called:

    // 1. Return "UTC".
    "UTC".to_owned()

    // TO-DO: full, system-aware implementation (and intl feature)
}

/// Abstract operation `CreateTemporalTimeZone ( identifier [ , newTarget ] )`
///
/// More information:
///  - [ECMAScript specififcation][spec]
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-createtemporaltimezone
#[allow(clippy::needless_pass_by_value, unused)]
pub(super) fn create_temporal_time_zone(
    identifier: String,
    new_target: Option<JsValue>,
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    // 1. If newTarget is not present, set newTarget to %Temporal.TimeZone%.
    let new_target = new_target.unwrap_or_else(|| todo!("%Temporal.TimeZone%"));

    // 2. Let object be ? OrdinaryCreateFromConstructor(newTarget, "%Temporal.TimeZone.prototype%", « [[InitializedTemporalTimeZone]], [[Identifier]], [[OffsetNanoseconds]] »).
    let prototype =
        get_prototype_from_constructor(&new_target, StandardConstructors::time_zone, context)?;

    // 3. Let offsetNanosecondsResult be Completion(ParseTimeZoneOffsetString(identifier)).
    let offset_nanoseconds_result = parse_timezone_offset_string(&identifier, context);

    // 4. If offsetNanosecondsResult is an abrupt completion, then
    let (identifier, offset_nanoseconds) = if let Ok(offset_nanoseconds) = offset_nanoseconds_result
    {
        // Switched conditions for more idiomatic rust code structuring
        // 5. Else,
        // a. Set object.[[Identifier]] to ! FormatTimeZoneOffsetString(offsetNanosecondsResult.[[Value]]).
        // b. Set object.[[OffsetNanoseconds]] to offsetNanosecondsResult.[[Value]].
        (
            format_time_zone_offset_string(offset_nanoseconds),
            Some(offset_nanoseconds),
        )
    } else {
        // a. Assert: ! CanonicalizeTimeZoneName(identifier) is identifier.
        assert_eq!(canonicalize_time_zone_name(&identifier), identifier);

        // b. Set object.[[Identifier]] to identifier.
        // c. Set object.[[OffsetNanoseconds]] to undefined.
        (identifier, None)
    };

    // 6. Return object.
    let object = JsObject::from_proto_and_data(
        prototype,
        ObjectData::time_zone(TimeZone {
            initialized_temporal_time_zone: false,
            identifier,
            offset_nanoseconds,
        }),
    );
    Ok(object.into())
}

/// Abstract operation `ParseTimeZoneOffsetString ( offsetString )`
///
/// The abstract operation `ParseTimeZoneOffsetString` takes argument `offsetString` (a String). It
/// parses the argument as a numeric UTC offset string and returns a signed integer representing
/// that offset as a number of nanoseconds.
///
/// More information:
///  - [ECMAScript specififcation][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-parsetimezoneoffsetstring
#[allow(clippy::unnecessary_wraps, unused)]
fn parse_timezone_offset_string(offset_string: &str, context: &mut Context<'_>) -> JsResult<i64> {
    use boa_parser::parser::{Cursor, TokenParser};

    // 1. Let parseResult be ParseText(StringToCodePoints(offsetString), UTCOffset).
    let parse_result = UTCOffset
        .parse(
            &mut Cursor::new(offset_string.as_bytes()),
            context.interner_mut(),
        )
        // 2. Assert: parseResult is not a List of errors.
        .expect("must not fail as per the spec");

    // 3. Assert: parseResult contains a TemporalSign Parse Node.

    // 4. Let parsedSign be the source text matched by the TemporalSign Parse Node contained within
    //    parseResult.
    // 5. If parsedSign is the single code point U+002D (HYPHEN-MINUS) or U+2212 (MINUS SIGN), then
    let sign = if matches!(parse_result.sign, OffsetSign::Negative) {
        // a. Let sign be -1.
        -1
    } else {
        // 6. Else,
        // a. Let sign be 1.
        1
    };

    // 7. NOTE: Applications of StringToNumber below do not lose precision, since each of the parsed
    //    values is guaranteed to be a sufficiently short string of decimal digits.
    // 8. Assert: parseResult contains an Hour Parse Node.
    // 9. Let parsedHours be the source text matched by the Hour Parse Node contained within parseResult.
    let parsed_hours = parse_result.hour;

    // 10. Let hours be ℝ(StringToNumber(CodePointsToString(parsedHours))).
    // 11. If parseResult does not contain a MinuteSecond Parse Node, then
    // a. Let minutes be 0.
    // 12. Else,
    // a. Let parsedMinutes be the source text matched by the first MinuteSecond Parse Node contained within parseResult.
    // b. Let minutes be ℝ(StringToNumber(CodePointsToString(parsedMinutes))).
    // 13. If parseResult does not contain two MinuteSecond Parse Nodes, then
    // a. Let seconds be 0.
    // 14. Else,
    // a. Let parsedSeconds be the source text matched by the second MinuteSecond Parse Node contained within parseResult.
    // b. Let seconds be ℝ(StringToNumber(CodePointsToString(parsedSeconds))).
    // 15. If parseResult does not contain a TemporalDecimalFraction Parse Node, then
    // a. Let nanoseconds be 0.
    // 16. Else,
    // a. Let parsedFraction be the source text matched by the TemporalDecimalFraction Parse Node contained within parseResult.
    // b. Let fraction be the string-concatenation of CodePointsToString(parsedFraction) and "000000000".
    // c. Let nanosecondsString be the substring of fraction from 1 to 10.
    // d. Let nanoseconds be ℝ(StringToNumber(nanosecondsString)).
    // 17. Return sign × (((hours × 60 + minutes) × 60 + seconds) × 10^9 + nanoseconds).

    todo!()
}

/// Abstract operation `FormatTimeZoneOffsetString ( offsetNanoseconds )`
fn format_time_zone_offset_string(offset_nanoseconds: i64) -> String {
    // 1. Assert: offsetNanoseconds is an integer.

    // 2. If offsetNanoseconds ≥ 0, let sign be "+"; otherwise, let sign be "-".
    let sign = if offset_nanoseconds >= 0 { "+" } else { "-" };

    // 3. Let offsetNanoseconds be abs(offsetNanoseconds).
    let offset_nanoseconds = offset_nanoseconds.unsigned_abs();

    // 4. Let nanoseconds be offsetNanoseconds modulo 10^9.
    let nanoseconds = offset_nanoseconds % 1_000_000_000;

    // 5. Let seconds be floor(offsetNanoseconds / 10^9) modulo 60.
    let seconds = (offset_nanoseconds / 1_000_000_000) % 60;

    // 6. Let minutes be floor(offsetNanoseconds / (6 × 10^10)) modulo 60.
    let minutes = (offset_nanoseconds / 60_000_000_000) % 60;

    // 7. Let hours be floor(offsetNanoseconds / (3.6 × 1012)).
    let hours = (offset_nanoseconds / 3_600_000_000_000) % 60;

    // 8. Let h be ToZeroPaddedDecimalString(hours, 2).
    let h = to_zero_padded_decimal_string(hours, 2);

    // 9. Let m be ToZeroPaddedDecimalString(minutes, 2).
    let m = to_zero_padded_decimal_string(minutes, 2);

    // 10. Let s be ToZeroPaddedDecimalString(seconds, 2).
    let s = to_zero_padded_decimal_string(seconds, 2);

    // 11. If nanoseconds ≠ 0, then
    let post = if nanoseconds != 0 {
        // a. Let fraction be ToZeroPaddedDecimalString(nanoseconds, 9).
        let fraction = to_zero_padded_decimal_string(nanoseconds, 9);

        // b. Set fraction to the longest possible substring of fraction starting at position 0 and not ending with the code unit 0x0030 (DIGIT ZERO).
        let fraction = fraction.trim_end_matches('0');

        // c. Let post be the string-concatenation of the code unit 0x003A (COLON), s, the code unit 0x002E (FULL STOP), and fraction.
        format!(":{s}.{fraction}")
    } else if seconds != 0 {
        // 12. Else if seconds ≠ 0, then
        // a. Let post be the string-concatenation of the code unit 0x003A (COLON) and s.
        format!(":{s}")
    } else {
        // 13. Else,
        // a. Let post be the empty String.
        String::new()
    };

    // 14. Return the string-concatenation of sign, h, the code unit 0x003A (COLON), m, and post.
    format!("{sign}{h}:{m}{post}")
}

/// Abstract operation `CanonicalizeTimeZoneName ( timeZone )`
///
/// The abstract operation `CanonicalizeTimeZoneName` takes argument `timeZone` (a String that is a
/// valid time zone name as verified by `IsAvailableTimeZoneName`). It returns the canonical and
/// case-regularized form of `timeZone`.
fn canonicalize_time_zone_name(time_zone: &str) -> String {
    // The minimum implementation of CanonicalizeTimeZoneName for ECMAScript implementations that
    // do not include local political rules for any time zones performs the following steps when
    // called:
    // 1. Assert: timeZone is an ASCII-case-insensitive match for "UTC".
    assert!(time_zone.to_ascii_uppercase() == "UTC");
    // 2. Return "UTC".
    "UTC".to_owned()
}
