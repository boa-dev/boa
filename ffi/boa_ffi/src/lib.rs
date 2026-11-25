//! C bindings for the Boa JavaScript engine.
//!
//! The C API and its ownership rules are documented in the crate's README.
//! Rust applications should use `boa_engine` directly.
#![cfg_attr(
    not(test),
    forbid(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)
)]

/// Entry points used by the generated C bindings.
#[diplomat::bridge]
#[diplomat::abi_rename = "boa_{0}"]
#[allow(
    clippy::unnecessary_box_returns,
    reason = "Diplomat transfers opaque handle ownership to C through boxed pointers"
)]
pub mod ffi {
    use std::fmt::Write as _;

    use boa_engine::{
        Context, JsError as BoaJsError, JsNativeError as BoaJsNativeError,
        JsNativeErrorKind as BoaJsNativeErrorKind, JsString as BoaJsString, JsValue as BoaJsValue,
        context::ContextBuilder, value::Type,
    };
    use diplomat_runtime::{DiplomatOption, DiplomatStr, DiplomatUtf8StrSlice, DiplomatWrite};

    /// An owned Boa context, exposed as an opaque C handle.
    ///
    /// Keep the handle on the thread that created it. Destroy it exactly once
    /// with `boa_BoaContext_destroy`; copying the pointer does not copy ownership.
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

    /// The reason a value could not be read as a UTF-8 string.
    #[derive(Debug, PartialEq, Eq)]
    pub enum JsStringError {
        /// The value is not a string.
        NotAString,
        /// The string contains an unpaired UTF-16 surrogate, so it has no UTF-8 form.
        InvalidEncoding,
    }

    /// An owned JavaScript value, exposed as an opaque C handle.
    ///
    /// Keep it on its creating thread and destroy it exactly once with
    /// `boa_JsValue_destroy`. Getters borrow a live, non-null handle without
    /// transferring ownership. Do not call `free` on it.
    #[derive(Debug)]
    #[diplomat::opaque]
    pub struct JsValue(BoaJsValue);

    impl JsValue {
        /// Creates an owned boolean value without requiring a context.
        #[must_use]
        pub fn from_boolean(value: bool) -> Box<JsValue> {
            Box::new(Self(BoaJsValue::from(value)))
        }

        /// Creates an owned number value without requiring a context.
        #[must_use]
        pub fn from_number(number: f64) -> Box<JsValue> {
            Box::new(Self(BoaJsValue::from(number)))
        }

        /// Creates an owned string value by copying UTF-8 bytes, including any NULs.
        ///
        /// Returns null in C if the input is not valid UTF-8. Empty input may
        /// pass a null pointer with length 0. No context is required. Allocation
        /// failures and unexpected panics may abort.
        #[must_use]
        pub fn from_string(value: &DiplomatStr) -> Option<Box<JsValue>> {
            let text = std::str::from_utf8(value).ok()?;
            Some(Box::new(Self(BoaJsValue::from(BoaJsString::from(text)))))
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

        /// Returns the boolean, or an absent result for any other type.
        ///
        /// No conversion is performed; truthy and falsy non-boolean values return
        /// an absent result. In C, check `is_ok` before reading `ok`.
        #[must_use]
        pub fn as_boolean(&self) -> DiplomatOption<bool> {
            self.0.as_boolean().into()
        }

        /// Returns the number, or an absent result for any other type.
        ///
        /// No conversion is performed. In C, check `is_ok` before reading `ok`.
        /// `NaN` is a number and produces a present result.
        #[must_use]
        pub fn as_number(&self) -> DiplomatOption<f64> {
            self.0.as_number().into()
        }

        /// Writes this string as UTF-8 into `write`, without coercion.
        ///
        /// Embedded NULs are preserved. In C, check `is_ok` before reading from
        /// `write`. On error, nothing is written to `write`.
        ///
        /// # Errors
        ///
        /// Returns `NotAString` for any non-string value, or `InvalidEncoding`
        /// if the string contains an unpaired UTF-16 surrogate.
        pub fn as_string(&self, write: &mut DiplomatWrite) -> Result<(), JsStringError> {
            let Some(string) = self.0.as_string() else {
                return Err(JsStringError::NotAString);
            };
            let Ok(utf8) = string.to_std_string() else {
                return Err(JsStringError::InvalidEncoding);
            };
            // Write the entire string in one call so fixed buffers fail
            // atomically and `diplomat_buffer_write_create` grows at most once.
            let _ = write.write_str(&utf8);
            Ok(())
        }

        /// Writes this string as UTF-8 into `write`, replacing unpaired UTF-16
        /// surrogates with `U+FFFD`, or returns an absent result for any other type.
        ///
        /// No conversion is performed and embedded NULs are preserved. In C,
        /// check `is_ok` before reading from `write`.
        #[must_use]
        pub fn as_string_lossy(&self, write: &mut DiplomatWrite) -> Option<()> {
            let string = self.0.as_string()?;
            let utf8 = string.to_std_string_lossy();
            // Write the entire string in one call so fixed buffers fail
            // atomically and `diplomat_buffer_write_create` grows at most once.
            let _ = write.write_str(&utf8);
            Some(())
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
    /// `boa_JsError_destroy`. Getters borrow a live, non-null handle.
    #[derive(Debug)]
    #[diplomat::opaque]
    pub struct JsError(BoaJsError);

    impl JsError {
        /// Creates a native error, copying the UTF-8 message, including any NULs.
        ///
        /// Returns null in C for `Other` or malformed UTF-8. Aggregate errors start
        /// with no members. No context is required. Allocation failures and panics
        /// may abort.
        #[must_use]
        pub fn from_native(kind: JsNativeErrorKind, message: &DiplomatStr) -> Option<Box<JsError>> {
            let message = std::str::from_utf8(message).ok()?;
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
            reason = "Diplomat requires a named lifetime for this borrowed return type"
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
        /// with `boa_JsValue_destroy`, independently of this error. No JavaScript runs.
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
            BoaContext, BoaJsError, BoaJsNativeError, BoaJsNativeErrorKind, BoaJsString,
            BoaJsValue, Context, JsError, JsErrorRepresentation, JsNativeErrorKind, JsStringError,
            JsValue, JsValueType,
        };
        use boa_engine::{
            Source,
            error::{EngineError, PanicError, RuntimeLimitError},
            gc, js_string,
        };
        use diplomat_runtime::rust_interop::RustWriteVec;

        fn read_string(value: &JsValue) -> Result<Vec<u8>, JsStringError> {
            let mut buffer = RustWriteVec::with_capacity(0);
            // SAFETY: this is the only `DiplomatWrite` in scope, so it cannot be swapped.
            let result = value.as_string(unsafe { buffer.borrow_mut() });
            let bytes = buffer.borrow().as_bytes().to_vec();
            match result {
                Ok(()) => Ok(bytes),
                Err(error) => {
                    assert!(bytes.is_empty(), "failed as_string should not write bytes");
                    Err(error)
                }
            }
        }

        fn read_string_lossy(value: &JsValue) -> Option<Vec<u8>> {
            let mut buffer = RustWriteVec::with_capacity(0);
            // SAFETY: this is the only `DiplomatWrite` in scope, so it cannot be swapped.
            let result = value.as_string_lossy(unsafe { buffer.borrow_mut() });
            let bytes = buffer.borrow().as_bytes().to_vec();
            if result.is_none() {
                assert!(
                    bytes.is_empty(),
                    "failed as_string_lossy should not write bytes"
                );
            }
            result.map(|()| bytes)
        }

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
        fn booleans_round_trip_without_coercion() {
            for expected in [false, true] {
                let value = JsValue::from_boolean(expected);
                assert_eq!(value.get_type(), JsValueType::Boolean);
                assert_eq!(value.as_boolean().into_option(), Some(expected));
                assert!(value.as_number().into_option().is_none());
                assert_eq!(read_string(&value), Err(JsStringError::NotAString));
                assert!(read_string_lossy(&value).is_none());
            }

            // Truthy and falsy non-boolean values must not coerce.
            for value in [
                JsValue::from_number(0.0),
                JsValue::from_number(1.0),
                JsValue::from_number(1.5),
                JsValue::from_string(b"").expect("valid UTF-8"),
                JsValue::from_string(b"true").expect("valid UTF-8"),
            ] {
                assert!(value.as_boolean().into_option().is_none());
            }
        }

        #[test]
        fn strings_round_trip_and_reject_invalid_utf8() {
            for text in [
                "",
                "hello",
                "latin1 \u{00e9}",
                "crab \u{1f980}",
                "before\0after",
            ] {
                let value = JsValue::from_string(text.as_bytes()).expect("valid UTF-8");
                assert_eq!(value.get_type(), JsValueType::String, "{text:?}");
                assert!(value.as_boolean().into_option().is_none());
                assert!(value.as_number().into_option().is_none());
                assert_eq!(
                    read_string(&value),
                    Ok(text.as_bytes().to_vec()),
                    "{text:?}"
                );
                assert_eq!(
                    read_string_lossy(&value),
                    Some(text.as_bytes().to_vec()),
                    "{text:?}"
                );
            }

            for invalid in [&b"\xff"[..], b"\xed\xa0\x80"] {
                assert!(JsValue::from_string(invalid).is_none(), "{invalid:?}");
            }
        }

        #[test]
        fn unpaired_surrogates_fail_strict_and_replace_in_lossy() {
            for (units, expected_lossy) in [
                (&[0xD800u16][..], "\u{fffd}"),
                (&[0xDC00u16][..], "\u{fffd}"),
                (
                    &[u16::from(b'a'), 0xD800, u16::from(b'b')][..],
                    "a\u{fffd}b",
                ),
            ] {
                let value = JsValue(BoaJsValue::from(BoaJsString::from(units)));
                assert_eq!(value.get_type(), JsValueType::String, "{units:x?}");
                assert_eq!(
                    read_string(&value),
                    Err(JsStringError::InvalidEncoding),
                    "{units:x?}"
                );
                assert_eq!(
                    read_string_lossy(&value),
                    Some(expected_lossy.as_bytes().to_vec()),
                    "{units:x?}"
                );
            }
        }

        #[test]
        fn engine_produced_strings_read_across_representations() {
            let mut context = Context::builder()
                .build()
                .expect("context should initialize");
            for (source, expected) in [
                ("'hello world'.slice(6)", Ok(b"world".as_slice())),
                ("'a' + '\\0' + '\\u{1F980}'", Ok("a\0\u{1f980}".as_bytes())),
                ("'\\ud800'", Err(JsStringError::InvalidEncoding)),
            ] {
                let value = JsValue(
                    context
                        .eval(Source::from_bytes(source))
                        .expect("valid source"),
                );
                assert_eq!(value.get_type(), JsValueType::String, "{source}");
                assert_eq!(
                    read_string(&value),
                    expected.map(<[u8]>::to_vec),
                    "{source}"
                );
                if expected.is_err() {
                    assert_eq!(
                        read_string_lossy(&value),
                        Some("\u{fffd}".as_bytes().to_vec()),
                        "{source}"
                    );
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
                assert!(value.as_boolean().into_option().is_none());
                assert!(value.as_number().into_option().is_none());
                assert_eq!(read_string(&value), Err(JsStringError::NotAString));
                assert!(read_string_lossy(&value).is_none());
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
                    value.as_boolean().into_option().is_some(),
                    expected_type == JsValueType::Boolean,
                    "{source}"
                );
                assert_eq!(
                    value.as_number().into_option().is_some(),
                    expected_type == JsValueType::Number,
                    "{source}"
                );
                assert_eq!(
                    read_string(&value).is_ok(),
                    expected_type == JsValueType::String,
                    "{source}"
                );
                assert_eq!(
                    read_string_lossy(&value).is_some(),
                    expected_type == JsValueType::String,
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
                    let error =
                        JsError::from_native(kind, message.as_bytes()).expect("known category");
                    assert_eq!(error.get_representation(), JsErrorRepresentation::Native);
                    let info = error.as_native().expect("native details");
                    assert_eq!(info.kind, kind);
                    assert_eq!(info.message.as_bytes(), message.as_bytes());
                    assert_eq!(error.0.as_native().expect("native error").kind(), &expected);
                    assert!(error.as_opaque().is_none());
                }
            }
            assert!(JsError::from_native(JsNativeErrorKind::Other, b"reserved").is_none());
            for invalid in [&b"\xff"[..], b"bad \xf0\x9f\xa6", b"\xed\xa0\x80"] {
                assert!(JsError::from_native(JsNativeErrorKind::Type, invalid).is_none());
            }
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
