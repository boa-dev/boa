//! The ECMAScript `Temporal` stage 3 built-in implementation.
//!
//! More information:
//!
//! [spec]: https://tc39.es/proposal-temporal/
#![allow(unreachable_code, unused_imports)] // Unimplemented

mod duration;
mod instant;
mod now;
mod plain_date;
mod plain_date_time;
mod plain_month_day;
mod plain_time;
mod plain_year_month;
mod time_zone;

pub(crate) use self::{
    duration::*, instant::*, now::*, plain_date::*, plain_date_time::*, plain_month_day::*,
    plain_time::*, plain_year_month::*, time_zone::*,
};
use super::{BuiltInBuilder, BuiltInObject, IntrinsicObject};
use crate::{
    context::intrinsics::{Intrinsics, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, ObjectData, ObjectInitializer},
    property::Attribute,
    realm::Realm,
    value::IntegerOrInfinity,
    Context, JsNativeError, JsObject, JsResult, JsSymbol, JsValue, NativeFunction,
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
                Now::init(realm),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().temporal()
    }
}

// -- Temporal Abstract Operations --

/// Abstract operation `ToZeroPaddedDecimalString ( n, minLength )`
///
/// The abstract operation `ToZeroPaddedDecimalString` takes arguments `n` (a non-negative integer)
/// and `minLength` (a non-negative integer) and returns a String.
fn to_zero_padded_decimal_string(n: u64, min_length: usize) -> String {
    format!("{n:0min_length$}")
}

/// Abstract operation 13.45 `ToIntegerIfIntegral( argument )`
pub(crate) fn to_integer_if_integral(arg: &JsValue, context: &mut Context<'_>) -> JsResult<i32> {
    if !arg.is_integer() {
        return Err(JsNativeError::range()
            .with_message("value to convert is not an integral number.")
            .into());
    }

    let number = arg.to_i32(context)?;
    Ok(number)
}
