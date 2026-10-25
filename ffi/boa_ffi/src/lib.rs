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
    use boa_engine::{
        Context, JsError as BoaJsError, JsNativeError as BoaJsNativeError,
        JsNativeErrorKind as BoaJsNativeErrorKind, JsValue as BoaJsValue, context::ContextBuilder,
        value::Type,
    };
    use diplomat_runtime::{DiplomatOption, DiplomatUtf8StrSlice};

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
    #[diplomat::enum_convert(Type)]
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
            self.0.get_type().into()
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

    /// The stored representation of an error, without normalization.
    #[derive(Debug, PartialEq, Eq)]
    pub enum JsErrorRepresentation {
        /// A native Boa error with a category and message.
        Native,
        /// An arbitrary thrown JavaScript value, including Error objects.
        Opaque,
        /// An engine failure that JavaScript cannot catch.
        Engine,
    }

    /// The category of a native Boa error.
    #[derive(Debug, PartialEq, Eq)]
    pub enum JsNativeErrorKind {
        /// A generic error.
        Error,
        /// An evaluation error.
        Eval,
        /// A range error.
        Range,
        /// A reference error.
        Reference,
        /// A syntax error.
        Syntax,
        /// A type error.
        Type,
        /// A URI error.
        Uri,
        /// An aggregate error.
        Aggregate,
        /// A native category not recognized by these bindings; inspection only.
        Other,
    }

    /// Native error details borrowed from a live error handle.
    pub struct JsNativeErrorInfo<'a> {
        /// The native category.
        pub kind: JsNativeErrorKind,
        /// UTF-8 bytes, valid until the error is destroyed. Do not free them.
        pub message: DiplomatUtf8StrSlice<'a>,
    }

    /// An owned Boa error, retaining its original representation and data.
    ///
    /// Keep it on its creating thread and destroy it exactly once with
    /// `JsError_destroy`. Getters borrow a live, non-null handle.
    #[derive(Debug)]
    #[diplomat::opaque]
    pub struct JsError(BoaJsError);

    impl JsError {
        /// Creates a native error, copying the UTF-8 message, including any NULs.
        ///
        /// Returns null in C for `Other`. Aggregate errors start with no members.
        /// No context is required. Allocation failures and panics may abort.
        #[must_use]
        pub fn from_native(kind: JsNativeErrorKind, message: &str) -> Option<Box<JsError>> {
            let error = match kind {
                JsNativeErrorKind::Error => BoaJsNativeError::error(),
                JsNativeErrorKind::Eval => BoaJsNativeError::eval(),
                JsNativeErrorKind::Range => BoaJsNativeError::range(),
                JsNativeErrorKind::Reference => BoaJsNativeError::reference(),
                JsNativeErrorKind::Syntax => BoaJsNativeError::syntax(),
                JsNativeErrorKind::Type => BoaJsNativeError::typ(),
                JsNativeErrorKind::Uri => BoaJsNativeError::uri(),
                JsNativeErrorKind::Aggregate => BoaJsNativeError::aggregate(Vec::new()),
                JsNativeErrorKind::Other => return None,
            };
            let error = error.with_message(message.to_owned());
            Some(Box::new(Self(error.into())))
        }

        /// Retains a borrowed value in a new owned opaque error.
        ///
        /// The caller keeps the input handle. Object identity is preserved.
        #[must_use]
        pub fn from_opaque(value: &JsValue) -> Box<JsError> {
            Box::new(Self(BoaJsError::from_opaque(value.0.clone())))
        }

        /// Returns the stored representation without executing JavaScript.
        #[must_use]
        pub fn get_representation(&self) -> JsErrorRepresentation {
            if self.0.as_native().is_some() {
                JsErrorRepresentation::Native
            } else if self.0.as_opaque().is_some() {
                JsErrorRepresentation::Opaque
            } else {
                JsErrorRepresentation::Engine
            }
        }

        /// Borrows native details, or returns an absent result for other representations.
        ///
        /// In C, check `is_ok` before reading `ok`. The message view is valid only
        /// while this error is alive. No JavaScript properties are read.
        #[must_use]
        #[allow(
            clippy::elidable_lifetime_names,
            reason = "Diplomat 0.15 requires a named lifetime for this borrowed return type"
        )]
        pub fn as_native<'a>(&'a self) -> Option<JsNativeErrorInfo<'a>> {
            self.0.as_native().map(|error| JsNativeErrorInfo {
                kind: match error.kind() {
                    BoaJsNativeErrorKind::Error => JsNativeErrorKind::Error,
                    BoaJsNativeErrorKind::Eval => JsNativeErrorKind::Eval,
                    BoaJsNativeErrorKind::Range => JsNativeErrorKind::Range,
                    BoaJsNativeErrorKind::Reference => JsNativeErrorKind::Reference,
                    BoaJsNativeErrorKind::Syntax => JsNativeErrorKind::Syntax,
                    BoaJsNativeErrorKind::Type => JsNativeErrorKind::Type,
                    BoaJsNativeErrorKind::Uri => JsNativeErrorKind::Uri,
                    BoaJsNativeErrorKind::Aggregate(_) => JsNativeErrorKind::Aggregate,
                    _ => JsNativeErrorKind::Other,
                },
                message: error.message().into(),
            })
        }

        /// Returns a new owned handle for an opaque payload, or null in C otherwise.
        ///
        /// Null and undefined payloads are present. Destroy each returned handle
        /// with `JsValue_destroy`, independently of this error. No JavaScript runs.
        #[must_use]
        pub fn as_opaque(&self) -> Option<Box<JsValue>> {
            self.0
                .as_opaque()
                .map(|value| Box::new(JsValue(value.clone())))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{
            BoaContext, BoaJsError, BoaJsNativeError, BoaJsNativeErrorKind, BoaJsValue, Context,
            JsError, JsErrorRepresentation, JsNativeErrorKind, JsValue, JsValueType,
        };
        use boa_engine::{
            Source,
            error::{EngineError, PanicError, RuntimeLimitError},
            gc, js_string,
        };

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

        #[test]
        fn native_errors_preserve_categories_and_messages() {
            for (kind, expected) in [
                (JsNativeErrorKind::Error, BoaJsNativeErrorKind::Error),
                (JsNativeErrorKind::Eval, BoaJsNativeErrorKind::Eval),
                (JsNativeErrorKind::Range, BoaJsNativeErrorKind::Range),
                (
                    JsNativeErrorKind::Reference,
                    BoaJsNativeErrorKind::Reference,
                ),
                (JsNativeErrorKind::Syntax, BoaJsNativeErrorKind::Syntax),
                (JsNativeErrorKind::Type, BoaJsNativeErrorKind::Type),
                (JsNativeErrorKind::Uri, BoaJsNativeErrorKind::Uri),
                (
                    JsNativeErrorKind::Aggregate,
                    BoaJsNativeErrorKind::Aggregate(Vec::new()),
                ),
            ] {
                for message in ["", "invalid \u{1f980} input", "before\0after"] {
                    let error = JsError::from_native(kind, message).expect("known category");
                    assert_eq!(error.get_representation(), JsErrorRepresentation::Native);
                    let info = error.as_native().expect("native details");
                    assert_eq!(info.kind, kind);
                    assert_eq!(info.message.as_bytes(), message.as_bytes());
                    assert_eq!(error.0.as_native().expect("native error").kind(), &expected);
                    assert!(error.as_opaque().is_none());
                }
            }
            assert!(JsError::from_native(JsNativeErrorKind::Other, "reserved").is_none());
        }

        #[test]
        fn opaque_error_inspection_retains_objects_without_invoking_getters() {
            let mut context = Context::builder()
                .build()
                .expect("context should initialize");
            for expression in ["new TypeError('boom')", "({})"] {
                let source = format!(
                    "globalThis.inspections = 0;
                     (() => {{
                         const value = {expression};
                         Object.defineProperties(value, {{
                             name: {{ get() {{ inspections++; throw 1; }} }},
                             message: {{ get() {{ inspections++; throw 2; }} }},
                             cause: {{ get() {{ inspections++; throw 3; }} }},
                             payload: {{ value: 7 }}
                         }});
                         throw value;
                     }})()"
                );
                let thrown = context
                    .eval(Source::from_bytes(&source))
                    .expect_err("source should throw");
                let original = JsValue(thrown.as_opaque().expect("opaque throw").clone());
                let reconstructed = JsError::from_opaque(&original);
                assert!(reconstructed.as_native().is_none());
                drop(reconstructed);

                let error = JsError(thrown);
                assert_eq!(error.get_representation(), JsErrorRepresentation::Opaque);
                assert!(error.as_native().is_none());
                let payload = error.as_opaque().expect("retained payload");
                assert!(BoaJsValue::same_value(&payload.0, &original.0));
                drop(original);
                drop(error);
                gc::force_collect();

                let value = payload
                    .0
                    .as_object()
                    .expect("object should be retained")
                    .get(js_string!("payload"), &mut context)
                    .expect("data property should be readable");
                assert_eq!(value.as_number(), Some(7.0));
                let inspections = context
                    .eval(Source::from_bytes("inspections"))
                    .expect("counter should be readable");
                assert_eq!(inspections.as_number(), Some(0.0));
            }
        }

        #[test]
        fn native_errors_retain_causes_and_aggregate_members() {
            let mut context = Context::builder()
                .build()
                .expect("context should initialize");
            let value = context
                .eval(Source::from_bytes("({ payload: 7 })"))
                .expect("object should be created");
            let cause = BoaJsError::from_opaque(value);
            let error = JsError(
                BoaJsNativeError::aggregate(vec![cause.clone()])
                    .with_message("aggregate")
                    .with_cause(cause)
                    .into(),
            );
            gc::force_collect();

            let info = error.as_native().expect("native details");
            assert_eq!(info.kind, JsNativeErrorKind::Aggregate);
            assert_eq!(info.message.as_bytes(), b"aggregate");
            assert!(error.as_opaque().is_none());
            let native = error.0.as_native().expect("native error");
            let BoaJsNativeErrorKind::Aggregate(members) = native.kind() else {
                panic!("aggregate members should be retained");
            };
            assert_eq!(members.len(), 1);
            let cause = native.cause().expect("cause should be retained");
            assert_eq!(cause, &members[0]);
            let payload = cause
                .as_opaque()
                .expect("opaque cause")
                .as_object()
                .expect("object cause")
                .get(js_string!("payload"), &mut context)
                .expect("data property should be readable");
            assert_eq!(payload.as_number(), Some(7.0));
        }

        #[test]
        fn engine_errors_remain_distinct() {
            for original in [
                EngineError::from(RuntimeLimitError::LoopIteration),
                EngineError::from(RuntimeLimitError::Recursion),
                EngineError::from(RuntimeLimitError::StackSize),
                EngineError::from(
                    PanicError::new("engine panic")
                        .with_source(BoaJsNativeError::error().with_message("source")),
                ),
            ] {
                let error = JsError(original.clone().into());
                assert_eq!(error.get_representation(), JsErrorRepresentation::Engine);
                assert!(error.as_native().is_none());
                assert!(error.as_opaque().is_none());
                assert_eq!(error.0.as_engine(), Some(&original));
            }
        }
    }
}

impl std::fmt::Debug for ffi::JsNativeErrorInfo<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message: &str = self.message.into();
        f.debug_struct("JsNativeErrorInfo")
            .field("kind", &self.kind)
            .field("message", &message)
            .finish()
    }
}
