//! Macros related to `JsValue`.

/// Create a `JsValue` from a simple DSL that resembles JSON.
///
///
#[macro_export]
macro_rules! js_value {
    // Single pattern to simplify documentation.
    ($($v:tt)+) => {
        $crate::js_value_internal!($($v)+)
    };
}

/// Internal macro rules for js_value!.
#[macro_export]
#[doc(hidden)]
macro_rules! js_value_internal {
    ([ $( $expr: tt ),* $(,)? ], $ctx: ident) => {
        $crate::JsValue::new(
            $crate::object::JsArray::from_iter(
                vec![ $( js_value!( $expr, $ctx ) ),* ],
                $ctx,
            )
        )
    };

    ({ $( $k: literal: $v: tt ),* $(,)? }, $ctx: ident) => {
        {
            let o = $crate::JsObject::with_null_proto();
            $(
                o.set( $crate::js_string!($k), js_value!( $v, $ctx ), false, $ctx )
                 .expect("Cannot set property of object.");
            )*

            $crate::JsValue::from(o)
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
