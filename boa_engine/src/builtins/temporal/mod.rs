//! The ECMAScript `Temporal` stage 3 built-in implementation.
//!
//! More information:
//!
//! [spec]: https://tc39.es/proposal-temporal/
#![allow(unreachable_code, unused_imports)] // Unimplemented

mod duration;
mod instant;
mod plain_date;
mod plain_date_time;
mod plain_month_day;
mod plain_time;
mod plain_year_month;
mod time_zone;

pub(crate) use self::{
    duration::*, instant::*, plain_date::*, plain_date_time::*, plain_month_day::*, plain_time::*,
    plain_year_month::*, time_zone::*,
};
use super::{BuiltInBuilder, BuiltInObject, IntrinsicObject};
use crate::{
    context::intrinsics::{Intrinsics, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, ObjectData, ObjectInitializer},
    property::Attribute,
    realm::Realm,
    value::IntegerOrInfinity,
    Context, JsObject, JsResult, JsSymbol, JsValue, NativeFunction,
};
use boa_ast::temporal::{OffsetSign, UtcOffset};
use boa_profiler::Profiler;

/// The [`Temporal`][spec] builtin object.
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Temporal;

impl BuiltInObject for Temporal {
    const NAME: &'static str = "Temporal";
}

impl IntrinsicObject for Temporal {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .static_property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                "Now",
                TemporalNow::init(realm),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().temporal()
    }
}

/// JavaScript `Temporal.Now` object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct TemporalNow;

impl TemporalNow {
    const NAME: &'static str = "Temporal.Now";

    /// Initializes the `Temporal.Now` object.
    fn init(realm: &Realm) -> JsValue {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        // is an ordinary object.
        // has a [[Prototype]] internal slot whose value is %Object.prototype%.
        // is not a function object.
        // does not have a [[Construct]] internal method; it cannot be used as a constructor with the new operator.
        // does not have a [[Call]] internal method; it cannot be invoked as a function.
        ObjectInitializer::new(realm.clone())
            .property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .function(NativeFunction::from_fn_ptr(Self::time_zone), "timeZone", 0)
            // .function(Self::instant, "instant", 0)
            // .function(Self::plain_date_time, "plainDateTime", 2)
            // .function(Self::plain_date_time_iso, "plainDateTimeISO", 1)
            // .function(Self::zoned_date_time, "zonedDateTime", 2)
            // .function(Self::zoned_date_time_iso, "zonedDateTimeISO", 1)
            // .function(Self::plain_date, "plainDate", 2)
            // .function(Self::plain_date_iso, "plainDateISO", 1)
            // .function(Self::plain_time_iso, "plainTimeISO", 1)
            .build()
            .into()
    }

    /// `Temporal.Now.timeZone ( )`
    ///
    /// More information:
    ///  - [ECMAScript specififcation][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal.now.timezone
    #[allow(clippy::unnecessary_wraps)]
    fn time_zone(_: &JsValue, _args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Return ! SystemTimeZone().
        Ok(system_time_zone(context).expect("retrieving the system timezone must not fail"))
    }
}

/// Abstract operation `SystemTimeZone ( )`
///
/// More information:
///  - [ECMAScript specififcation][spec]
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-systemtimezone
#[allow(unused)]
fn system_time_zone(context: &mut Context<'_>) -> JsResult<JsValue> {
    // 1. Let identifier be ! DefaultTimeZone().
    let identifier = default_time_zone(context);
    // 2. Return ! CreateTemporalTimeZone(identifier).
    create_temporal_time_zone(identifier, None, context)
}

/// Abstract operation `ToZeroPaddedDecimalString ( n, minLength )`
///
/// The abstract operation `ToZeroPaddedDecimalString` takes arguments `n` (a non-negative integer)
/// and `minLength` (a non-negative integer) and returns a String.
fn to_zero_padded_decimal_string(n: u64, min_length: usize) -> String {
    format!("{n:0min_length$}")
}
