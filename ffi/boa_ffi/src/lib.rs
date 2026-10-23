//! C bindings for the Boa JavaScript engine.
//!
//! The C API and its ownership rules are documented in the crate's README.
//! Rust applications should use `boa_engine` directly.

/// Entry points used by the generated C bindings.
#[diplomat::bridge]
#[allow(
    clippy::unnecessary_box_returns,
    reason = "Diplomat transfers opaque handle ownership to C through boxed pointers"
)]
pub mod ffi {
    use boa_engine::{Context, JsValue as BoaJsValue, context::ContextBuilder, value::Type};
    use diplomat_runtime::DiplomatOption;

    /// An owned Boa context, exposed as an opaque C handle.
    ///
    /// Keep the handle on the thread that created it. Destroy it exactly once
    /// with `BoaContext_destroy`; copying the pointer does not copy ownership.
    /// The destructor requires a live, non-null handle. Do not call `free` on it.
    #[derive(Debug)]
    #[diplomat::opaque_mut]
    pub struct BoaContext {
        _context: Context,
    }

    impl BoaContext {
        /// Creates a context, or returns null if engine initialization fails.
        ///
        /// A successful call transfers ownership to the caller. Multiple contexts
        /// can coexist on one thread. Unexpected Rust panics and allocation
        /// failures may abort the process rather than return null.
        #[must_use]
        pub fn new() -> Option<Box<BoaContext>> {
            Self::from_builder(Context::builder())
        }

        fn from_builder(builder: ContextBuilder) -> Option<Box<Self>> {
            builder
                .build()
                .ok()
                .map(|context| Box::new(Self { _context: context }))
        }
    }

    /// The ECMAScript language type of a value.
    ///
    /// Unlike JavaScript's `typeof`, this distinguishes null from objects.
    /// Functions have the object type.
    #[derive(Debug, PartialEq, Eq)]
    pub enum JsValueType {
        /// The undefined type.
        Undefined,
        /// The null type.
        Null,
        /// The boolean type.
        Boolean,
        /// The number type.
        Number,
        /// The string type.
        String,
        /// The symbol type.
        Symbol,
        /// The bigint type.
        BigInt,
        /// The object type, including functions.
        Object,
    }

    /// An owned JavaScript value, exposed as an opaque C handle.
    ///
    /// Keep it on its creating thread and destroy it exactly once with
    /// `JsValue_destroy`. Getters borrow a live, non-null handle without
    /// transferring ownership. Do not call `free` on it.
    #[derive(Debug)]
    #[diplomat::opaque]
    pub struct JsValue(BoaJsValue);

    impl JsValue {
        /// Creates an owned number value without requiring a context.
        #[must_use]
        pub fn from_number(number: f64) -> Box<JsValue> {
            Box::new(Self(BoaJsValue::from(number)))
        }

        /// Creates an owned null value without requiring a context.
        #[must_use]
        pub fn from_null() -> Box<JsValue> {
            Box::new(Self(BoaJsValue::null()))
        }

        /// Creates an owned undefined value without requiring a context.
        #[must_use]
        pub fn from_undefined() -> Box<JsValue> {
            Box::new(Self(BoaJsValue::undefined()))
        }

        /// Returns the language type without conversion or JavaScript execution.
        #[must_use]
        pub fn get_type(&self) -> JsValueType {
            match self.0.get_type() {
                Type::Undefined => JsValueType::Undefined,
                Type::Null => JsValueType::Null,
                Type::Boolean => JsValueType::Boolean,
                Type::Number => JsValueType::Number,
                Type::String => JsValueType::String,
                Type::Symbol => JsValueType::Symbol,
                Type::BigInt => JsValueType::BigInt,
                Type::Object => JsValueType::Object,
            }
        }

        /// Returns the number, or an absent result for any other type.
        ///
        /// No conversion is performed. In C, check `is_ok` before reading `ok`.
        /// `NaN` is a number and produces a present result.
        #[must_use]
        pub fn as_number(&self) -> DiplomatOption<f64> {
            self.0.as_number().into()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{BoaContext, BoaJsValue, Context, JsValue, JsValueType};
        use boa_engine::{Source, gc, js_string};

        #[test]
        fn initialization_failure_returns_none() {
            let context = BoaContext::new().expect("default context should initialize");

            // A blocking context cannot share a thread with this live context.
            assert!(BoaContext::from_builder(Context::builder().can_block(true)).is_none());

            drop(context);

            // Destruction must release the engine's per-thread context state.
            let blocking = BoaContext::from_builder(Context::builder().can_block(true))
                .expect("blocking context should initialize after the conflict is gone");
            drop(blocking);
        }

        #[test]
        fn contexts_can_coexist_and_be_destroyed_in_any_order() {
            let first = BoaContext::new().expect("first context should initialize");
            let second = BoaContext::new().expect("second context should initialize");
            drop(first);

            let third = BoaContext::new().expect("third context should initialize");
            drop(second);
            drop(third);

            let blocking = Context::builder()
                .can_block(true)
                .build()
                .expect("all nonblocking contexts should have been released");
            drop(blocking);
        }

        #[test]
        fn numbers_round_trip_without_coercion() {
            for number in [
                0.0,
                -0.0,
                7.0,
                -7.25,
                f64::from(i32::MAX),
                f64::from(i32::MAX) + 1.0,
                f64::from(i32::MIN),
                f64::from(i32::MIN) - 1.0,
                9_007_199_254_740_991.0,
                f64::MAX,
                f64::MIN_POSITIVE,
                f64::from_bits(1),
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NAN,
            ] {
                let value = JsValue::from_number(number);
                assert_eq!(value.get_type(), JsValueType::Number);
                let actual = value.as_number().into_option().expect("number is present");
                if number.is_nan() {
                    assert!(actual.is_nan());
                } else {
                    assert_eq!(actual.to_bits(), number.to_bits(), "{number:?}");
                }
            }
        }

        #[test]
        fn null_and_undefined_are_distinct_non_numbers() {
            for (value, expected_type) in [
                (JsValue::from_null(), JsValueType::Null),
                (JsValue::from_undefined(), JsValueType::Undefined),
            ] {
                assert_eq!(value.get_type(), expected_type);
                assert!(value.as_number().into_option().is_none());
            }
        }

        #[test]
        fn type_query_handles_every_boa_value_type() {
            let mut context = Context::builder()
                .build()
                .expect("context should initialize");
            for (source, expected_type) in [
                ("undefined", JsValueType::Undefined),
                ("null", JsValueType::Null),
                ("true", JsValueType::Boolean),
                ("7", JsValueType::Number),
                ("'text'", JsValueType::String),
                ("Symbol('test')", JsValueType::Symbol),
                ("7n", JsValueType::BigInt),
                ("({})", JsValueType::Object),
                ("(() => 7)", JsValueType::Object),
            ] {
                let value = JsValue(
                    context
                        .eval(Source::from_bytes(source))
                        .expect("valid source"),
                );
                assert_eq!(value.get_type(), expected_type, "{source}");
                assert_eq!(
                    value.as_number().into_option().is_some(),
                    expected_type == JsValueType::Number,
                    "{source}"
                );
            }
        }

        #[test]
        fn retained_object_keeps_its_identity_and_contents() {
            let mut context = Context::builder()
                .build()
                .expect("context should initialize");
            let original = JsValue(
                context
                    .eval(Source::from_bytes("({ payload: 7 })"))
                    .expect("object should be created"),
            );
            let retained = Box::new(JsValue(original.0.clone()));
            assert!(BoaJsValue::same_value(&original.0, &retained.0));
            drop(original);
            gc::force_collect();

            assert_eq!(retained.get_type(), JsValueType::Object);
            assert!(retained.as_number().into_option().is_none());
            let payload = retained
                .0
                .as_object()
                .expect("object should be retained")
                .get(js_string!("payload"), &mut context)
                .expect("property should be readable");
            assert_eq!(payload.as_number(), Some(7.0));
        }
    }
}
