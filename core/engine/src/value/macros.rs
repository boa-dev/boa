//! Macros related to `JsValue`.

/// Create a `JsObject` object from a simpler DSL that resembles JSON.
///
/// ```
/// # use boa_engine::{js_string, js_object, Context};
/// # let context = &mut Context::default();
/// let value = js_object!({
///   // Comments are allowed inside.
///   "key": (js_string!("value"))
/// }, context);
/// ```
#[macro_export]
macro_rules! js_object {
    ({ $( $k: literal: $v: tt ),* $(,)? }, $ctx: ident) => {
        {
            let o = $crate::JsObject::with_null_proto();
            $(
                o.set( $crate::js_string!($k), $crate::js_value!( $v, $ctx ), false, $ctx )
                 .expect("Cannot set property of object.");
            )*

            o
        }
    };
}

/// Create a `JsValue` from a simple DSL that resembles JSON.
///
/// ```
/// # use boa_engine::{js_string, js_value, Context, JsValue};
/// # let context = &mut Context::default();
/// assert_eq!(js_value!( 1 ), JsValue::from(1));
/// assert_eq!(js_value!( false ), JsValue::from(false));
/// // Objects and arrays cannot be compared with simple equality.
/// // To create arrays and objects, the context needs to be passed in.
/// assert_eq!(js_value!([ 1, 2, 3 ], context).display().to_string(), "[ 1, 2, 3 ]");
///
/// assert_eq!(
///   js_value!({
///     // Comments are allowed inside.
///     "key": (js_string!("value"))
///   }, context).display().to_string(),
///   "{\n key: \"value\"\n}",
/// );
/// ```
#[macro_export]
macro_rules! js_value {
    // Single pattern to simplify documentation.
    ($($v:tt)+) => {
        $crate::js_value_internal!($($v)+)
    };
}

/// Internal macro rules for js_value!.
// TODO: move this to a proc_macro which can be distinguish between string and number literal.
#[macro_export]
#[doc(hidden)]
macro_rules! js_value_internal {
    ([ $( $expr: tt ),* $(,)? ], $ctx: ident) => {
        $crate::JsValue::new(
            $crate::object::builtins::JsArray::from_iter(
                vec![ $( $crate::js_value!( $expr, $ctx ) ),* ],
                $ctx,
            )
        )
    };

    ({ $( $k: literal: $v: tt ),* $(,)? }, $ctx: ident) => {
        {
            $crate::JsValue::from( $crate::js_object!({ $( $k: $v ),* }, $ctx) )
        }
    };

    // These are duplicated so we can match with context on those too.
    ($v: literal) => { $crate::JsValue::new($v) };
    ($v: expr) => { $crate::JsValue::new($v) };
    ($v: ident) => { $crate::JsValue::new($v) };
    ($v: tt) => { $crate::JsValue::new($v) };

    ($v: literal, $_: ident) => { $crate::JsValue::new($v) };
    ($v: expr, $_: ident) => { $crate::JsValue::new($v) };
    ($v: ident, $_: ident) => { $crate::JsValue::new($v) };
    ($v: tt, $_: ident) => { $crate::JsValue::new($v) };
}
