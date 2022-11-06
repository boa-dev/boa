//! Error-related types and conversions.

use crate::{
    builtins::{error::ErrorKind, Array},
    object::JsObject,
    object::ObjectData,
    property::PropertyDescriptor,
    Context, JsString, JsValue,
};
use boa_gc::{Finalize, Trace};
use thiserror::Error;

/// The error type returned by all operations related
/// to the execution of Javascript code.
///
/// This is essentially an enum that can store either [`JsNativeError`]s (for ideal
/// native errors)  or opaque [`JsValue`]s, since Javascript allows throwing any valid
/// `JsValue`.
///
/// The implementation doesn't provide a [`From`] conversion
/// for `JsValue`. This is with the intent of encouraging the usage of proper
/// `JsNativeError`s instead of plain `JsValue`s. However, if you
/// do need a proper opaque error, you can construct one using the
/// [`JsError::from_opaque`] method.
///
/// # Examples
///
/// ```rust
/// # use boa_engine::{JsError, JsNativeError, JsNativeErrorKind, JsValue};
/// let cause = JsError::from_opaque("error!".into());
///
/// assert!(cause.as_opaque().is_some());
/// assert_eq!(cause.as_opaque().unwrap(), &JsValue::from("error!"));
///
/// let native_error: JsError = JsNativeError::typ()
///     .with_message("invalid type!")
///     .with_cause(cause)
///     .into();
///
/// assert!(native_error.as_native().is_some());
///
/// let kind = &native_error.as_native().unwrap().kind;
/// assert!(matches!(kind, JsNativeErrorKind::Type));
/// ```
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsError {
    inner: Repr,
}

/// Internal representation of a [`JsError`].
///
/// `JsError` is represented by an opaque enum because it restricts
/// matching against `JsError` without calling `try_native` first.
/// This allows us to provide a safe API for `Error` objects that extracts
/// their info as a native `Rust` type ([`JsNativeError`]).
///
/// This should never be used outside of this module. If that's not the case,
/// you should add methods to either `JsError` or `JsNativeError` to
/// represent that special use case.
#[derive(Debug, Clone, Trace, Finalize)]
enum Repr {
    Native(JsNativeError),
    Opaque(JsValue),
}

/// The error type returned by the [`JsError::try_native`] method.
#[derive(Debug, Clone, Error)]
pub enum TryNativeError {
    #[error("invalid type of property `{0}`")]
    InvalidPropertyType(&'static str),
    #[error("property `message` cannot contain unpaired surrogates")]
    InvalidMessageEncoding,
    #[error("could not access property `{property}`")]
    InaccessibleProperty {
        property: &'static str,
        source: JsError,
    },
    #[error("could not get element `{index}` of property `errors`")]
    InvalidErrorsIndex { index: u64, source: JsError },
    #[error("opaque error of type `{:?}` is not an Error object", .0.get_type())]
    NotAnErrorObject(JsValue),
}

impl std::error::Error for JsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.inner {
            Repr::Native(err) => err.source(),
            Repr::Opaque(_) => None,
        }
    }
}

impl JsError {
    /// Creates a new `JsError` from a native error `err`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use boa_engine::{JsError, JsNativeError};
    /// let error = JsError::from_native(JsNativeError::syntax());
    ///
    /// assert!(error.as_native().is_some());
    /// ```
    pub fn from_native(err: JsNativeError) -> Self {
        Self {
            inner: Repr::Native(err),
        }
    }

    /// Creates a new `JsError` from an opaque error `value`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use boa_engine::JsError;
    /// let error = JsError::from_opaque(5.0f64.into());
    ///
    /// assert!(error.as_opaque().is_some());
    /// ```
    pub fn from_opaque(value: JsValue) -> Self {
        Self {
            inner: Repr::Opaque(value),
        }
    }

    /// Converts the error to an opaque `JsValue` error
    ///
    /// Unwraps the inner `JsValue` if the error is already an opaque error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use boa_engine::{Context, JsError, JsNativeError};
    /// let context = &mut Context::default();
    /// let error: JsError = JsNativeError::eval().with_message("invalid script").into();
    /// let error_val = error.to_opaque(context);
    ///
    /// assert!(error_val.as_object().unwrap().borrow().is_error());
    /// ```
    pub fn to_opaque(&self, context: &mut Context) -> JsValue {
        match &self.inner {
            Repr::Native(e) => e.to_opaque(context).into(),
            Repr::Opaque(v) => v.clone(),
        }
    }

    /// Unwraps the inner error if this contains a native error.
    /// Otherwise, inspects the opaque error and tries to extract the
    /// necessary information to construct a native error similar to the provided
    /// opaque error. If the conversion fails, returns a [`TryNativeError`]
    /// with the cause of the failure.
    ///
    /// # Note 1
    ///
    /// This method won't try to make any conversions between JS types.
    /// In other words, for this conversion to succeed:
    /// - `message` **MUST** be a `JsString` value.
    /// - `errors` (in the case of `AggregateError`s) **MUST** be an `Array` object.
    ///
    /// # Note 2
    ///
    /// This operation should be considered a lossy conversion, since it
    /// won't store any additional properties of the opaque
    /// error, other than `message`, `cause` and `errors` (in the case of
    /// `AggregateError`s). If you cannot affort a lossy conversion, clone
    /// the object before calling [`from_opaque`][JsError::from_opaque]
    /// to preserve its original properties.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use boa_engine::{Context, JsError, JsNativeError, JsNativeErrorKind};
    /// let context = &mut Context::default();
    ///
    /// // create a new, opaque Error object
    /// let error: JsError = JsNativeError::typ().with_message("type error!").into();
    /// let error_val = error.to_opaque(context);
    ///
    /// // then, try to recover the original
    /// let error = JsError::from_opaque(error_val).try_native(context).unwrap();
    ///
    /// assert!(matches!(error.kind, JsNativeErrorKind::Type));
    /// assert_eq!(error.message(), "type error!");
    /// ```
    pub fn try_native(&self, context: &mut Context) -> Result<JsNativeError, TryNativeError> {
        match &self.inner {
            Repr::Native(e) => Ok(e.clone()),
            Repr::Opaque(val) => {
                let obj = val
                    .as_object()
                    .ok_or_else(|| TryNativeError::NotAnErrorObject(val.clone()))?;
                let error = obj
                    .borrow()
                    .as_error()
                    .ok_or_else(|| TryNativeError::NotAnErrorObject(val.clone()))?;

                let try_get_property = |key, context: &mut Context| {
                    obj.has_property(key, context)
                        .map_err(|e| TryNativeError::InaccessibleProperty {
                            property: key,
                            source: e,
                        })?
                        .then(|| obj.get(key, context))
                        .transpose()
                        .map_err(|e| TryNativeError::InaccessibleProperty {
                            property: key,
                            source: e,
                        })
                };

                let message = if let Some(msg) = try_get_property("message", context)? {
                    msg.as_string()
                        .map(JsString::to_std_string)
                        .transpose()
                        .map_err(|_| TryNativeError::InvalidMessageEncoding)?
                        .ok_or(TryNativeError::InvalidPropertyType("message"))?
                        .into()
                } else {
                    Box::default()
                };

                let cause = try_get_property("cause", context)?;

                let kind = match error {
                    ErrorKind::Error => JsNativeErrorKind::Error,
                    ErrorKind::Eval => JsNativeErrorKind::Eval,
                    ErrorKind::Type => JsNativeErrorKind::Type,
                    ErrorKind::Range => JsNativeErrorKind::Range,
                    ErrorKind::Reference => JsNativeErrorKind::Reference,
                    ErrorKind::Syntax => JsNativeErrorKind::Syntax,
                    ErrorKind::Uri => JsNativeErrorKind::Uri,
                    ErrorKind::Aggregate => {
                        let errors = obj.get("errors", context).map_err(|e| {
                            TryNativeError::InaccessibleProperty {
                                property: "errors",
                                source: e,
                            }
                        })?;
                        let mut error_list = Vec::new();
                        match errors.as_object() {
                            Some(errors) if errors.is_array() => {
                                let length = errors.length_of_array_like(context).map_err(|e| {
                                    TryNativeError::InaccessibleProperty {
                                        property: "errors.length",
                                        source: e,
                                    }
                                })?;
                                for i in 0..length {
                                    error_list.push(JsError::from_opaque(
                                        errors.get(i, context).map_err(|e| {
                                            TryNativeError::InvalidErrorsIndex {
                                                index: i,
                                                source: e,
                                            }
                                        })?,
                                    ));
                                }
                            }
                            _ => return Err(TryNativeError::InvalidPropertyType("errors")),
                        }

                        JsNativeErrorKind::Aggregate(error_list)
                    }
                };

                Ok(JsNativeError {
                    kind,
                    message,
                    cause: cause.map(|v| Box::new(JsError::from_opaque(v))),
                })
            }
        }
    }

    /// Gets the inner [`JsValue`] if the error is an opaque error,
    /// or `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use boa_engine::{JsError, JsNativeError};
    /// let error: JsError = JsNativeError::reference()
    ///     .with_message("variable not found!")
    ///     .into();
    ///
    /// assert!(error.as_opaque().is_none());
    ///
    /// let error = JsError::from_opaque(256u32.into());
    ///
    /// assert!(error.as_opaque().is_some());
    /// ```
    pub fn as_opaque(&self) -> Option<&JsValue> {
        match self.inner {
            Repr::Native(_) => None,
            Repr::Opaque(ref v) => Some(v),
        }
    }

    /// Gets the inner [`JsNativeError`] if the error is a native
    /// error, or `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use boa_engine::{JsError, JsNativeError, JsValue};
    /// let error: JsError = JsNativeError::error().with_message("Unknown error").into();
    ///
    /// assert!(error.as_native().is_some());
    ///
    /// let error = JsError::from_opaque(JsValue::undefined().into());
    ///
    /// assert!(error.as_native().is_none());
    /// ```
    pub fn as_native(&self) -> Option<&JsNativeError> {
        match self.inner {
            Repr::Native(ref e) => Some(e),
            Repr::Opaque(_) => None,
        }
    }
}

impl From<boa_parser::Error> for JsError {
    fn from(err: boa_parser::Error) -> Self {
        Self::from(JsNativeError::from(err))
    }
}

impl From<JsNativeError> for JsError {
    fn from(error: JsNativeError) -> Self {
        Self {
            inner: Repr::Native(error),
        }
    }
}

impl std::fmt::Display for JsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            Repr::Native(e) => e.fmt(f),
            Repr::Opaque(v) => v.display().fmt(f),
        }
    }
}

/// Native representation of an ideal `Error` object from Javascript.
///
/// This representation is more space efficient than its [`JsObject`] equivalent,
/// since it doesn't need to create a whole new `JsObject` to be instantiated.
/// Prefer using this over [`JsError`] when you don't need to throw
/// plain [`JsValue`]s as errors, or when you need to inspect the error type
/// of a `JsError`.
///
/// # Examples
///
/// ```rust
/// # use boa_engine::{JsNativeError, JsNativeErrorKind};
/// let native_error = JsNativeError::uri().with_message("cannot decode uri");
///
/// match native_error.kind {
///     JsNativeErrorKind::Uri => { /* handle URI error*/ }
///     _ => unreachable!(),
/// }
///
/// assert_eq!(native_error.message(), "cannot decode uri");
/// ```
#[derive(Debug, Clone, Trace, Finalize, Error)]
#[error("{kind}: {message}")]
pub struct JsNativeError {
    /// The kind of native error (e.g. `TypeError`, `SyntaxError`, etc.)
    pub kind: JsNativeErrorKind,
    message: Box<str>,
    #[source]
    cause: Option<Box<JsError>>,
}

impl JsNativeError {
    /// Creates a new `JsNativeError` from its `kind`, `message` and (optionally) its `cause`.
    fn new(kind: JsNativeErrorKind, message: Box<str>, cause: Option<Box<JsError>>) -> Self {
        Self {
            kind,
            message,
            cause,
        }
    }

    /// Creates a new `JsNativeError` of kind `AggregateError` from a list of [`JsError`]s, with
    /// empty `message` and undefined `cause`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use boa_engine::{JsNativeError, JsNativeErrorKind};
    /// let inner_errors = vec![
    ///     JsNativeError::typ().into(),
    ///     JsNativeError::syntax().into()
    /// ];
    /// let error = JsNativeError::aggregate(inner_errors);
    ///
    /// assert!(matches!(
    ///     error.kind,
    ///     JsNativeErrorKind::Aggregate(ref errors) if errors.len() == 2
    /// ));
    /// ```
    pub fn aggregate(errors: Vec<JsError>) -> Self {
        Self::new(JsNativeErrorKind::Aggregate(errors), Box::default(), None)
    }

    /// Creates a new `JsNativeError` of kind `Error`, with empty `message` and undefined `cause`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use boa_engine::{JsNativeError, JsNativeErrorKind};
    /// let error = JsNativeError::error();
    ///
    /// assert!(matches!(error.kind, JsNativeErrorKind::Error));
    /// ```
    pub fn error() -> Self {
        Self::new(JsNativeErrorKind::Error, Box::default(), None)
    }

    /// Creates a new `JsNativeError` of kind `EvalError`, with empty `message` and undefined `cause`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use boa_engine::{JsNativeError, JsNativeErrorKind};
    /// let error = JsNativeError::eval();
    ///
    /// assert!(matches!(error.kind, JsNativeErrorKind::Eval));
    /// ```
    pub fn eval() -> Self {
        Self::new(JsNativeErrorKind::Eval, Box::default(), None)
    }

    /// Creates a new `JsNativeError` of kind `RangeError`, with empty `message` and undefined `cause`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use boa_engine::{JsNativeError, JsNativeErrorKind};
    /// let error = JsNativeError::range();
    ///
    /// assert!(matches!(error.kind, JsNativeErrorKind::Range));
    /// ```
    pub fn range() -> Self {
        Self::new(JsNativeErrorKind::Range, Box::default(), None)
    }

    /// Creates a new `JsNativeError` of kind `ReferenceError`, with empty `message` and undefined `cause`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use boa_engine::{JsNativeError, JsNativeErrorKind};
    /// let error = JsNativeError::reference();
    ///
    /// assert!(matches!(error.kind, JsNativeErrorKind::Reference));
    /// ```
    pub fn reference() -> Self {
        Self::new(JsNativeErrorKind::Reference, Box::default(), None)
    }

    /// Creates a new `JsNativeError` of kind `SyntaxError`, with empty `message` and undefined `cause`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use boa_engine::{JsNativeError, JsNativeErrorKind};
    /// let error = JsNativeError::syntax();
    ///
    /// assert!(matches!(error.kind, JsNativeErrorKind::Syntax));
    /// ```
    pub fn syntax() -> Self {
        Self::new(JsNativeErrorKind::Syntax, Box::default(), None)
    }

    /// Creates a new `JsNativeError` of kind `TypeError`, with empty `message` and undefined `cause`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use boa_engine::{JsNativeError, JsNativeErrorKind};
    /// let error = JsNativeError::typ();
    ///
    /// assert!(matches!(error.kind, JsNativeErrorKind::Type));
    /// ```
    pub fn typ() -> Self {
        Self::new(JsNativeErrorKind::Type, Box::default(), None)
    }

    /// Creates a new `JsNativeError` of kind `UriError`, with empty `message` and undefined `cause`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use boa_engine::{JsNativeError, JsNativeErrorKind};
    /// let error = JsNativeError::uri();
    ///
    /// assert!(matches!(error.kind, JsNativeErrorKind::Uri));
    /// ```
    pub fn uri() -> Self {
        Self::new(JsNativeErrorKind::Uri, Box::default(), None)
    }

    /// Creates a new `JsNativeError` that indicates that the context hit its execution limit. This
    /// is only used in a fuzzing context.
    #[cfg(feature = "fuzz")]
    pub fn no_instructions_remain() -> Self {
        Self::new(
            JsNativeErrorKind::NoInstructionsRemain,
            Box::default(),
            None,
        )
    }

    /// Sets the message of this error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use boa_engine::JsNativeError;
    /// let error = JsNativeError::range().with_message("number too large");
    ///
    /// assert_eq!(error.message(), "number too large");
    /// ```
    #[must_use]
    pub fn with_message<S>(mut self, message: S) -> Self
    where
        S: Into<Box<str>>,
    {
        self.message = message.into();
        self
    }

    /// Sets the cause of this error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use boa_engine::JsNativeError;
    /// let cause = JsNativeError::syntax();
    /// let error = JsNativeError::error().with_cause(cause);
    ///
    /// assert!(error.cause().unwrap().as_native().is_some());
    /// ```
    #[must_use]
    pub fn with_cause<V>(mut self, cause: V) -> Self
    where
        V: Into<JsError>,
    {
        self.cause = Some(Box::new(cause.into()));
        self
    }

    /// Gets the `message` of this error.
    ///
    /// This is equivalent to the [`NativeError.prototype.message`][spec]
    /// property.
    ///
    /// [spec]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error/message
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use boa_engine::JsNativeError;
    /// let error = JsNativeError::range().with_message("number too large");
    ///
    /// assert_eq!(error.message(), "number too large");
    /// ```
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Gets the `cause` of this error.
    ///
    /// This is equivalent to the [`NativeError.prototype.cause`][spec]
    /// property.
    ///
    /// [spec]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error/cause
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use boa_engine::JsNativeError;
    /// let cause = JsNativeError::syntax();
    /// let error = JsNativeError::error().with_cause(cause);
    ///
    /// assert!(error.cause().unwrap().as_native().is_some());
    /// ```
    pub fn cause(&self) -> Option<&JsError> {
        self.cause.as_deref()
    }

    /// Converts this native error to its opaque representation as a [`JsObject`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use boa_engine::{Context, JsError, JsNativeError};
    /// let context = &mut Context::default();
    ///
    /// let error = JsNativeError::error().with_message("error!");
    /// let error_obj = error.to_opaque(context);
    ///
    /// assert!(error_obj.borrow().is_error());
    /// assert_eq!(error_obj.get("message", context).unwrap(), "error!".into())
    /// ```
    pub fn to_opaque(&self, context: &mut Context) -> JsObject {
        let Self {
            kind,
            message,
            cause,
        } = self;
        let constructors = context.intrinsics().constructors();
        let (prototype, tag) = match kind {
            JsNativeErrorKind::Aggregate(_) => (
                constructors.aggregate_error().prototype(),
                ErrorKind::Aggregate,
            ),
            JsNativeErrorKind::Error => (constructors.error().prototype(), ErrorKind::Error),
            JsNativeErrorKind::Eval => (constructors.eval_error().prototype(), ErrorKind::Eval),
            JsNativeErrorKind::Range => (constructors.range_error().prototype(), ErrorKind::Range),
            JsNativeErrorKind::Reference => (
                constructors.reference_error().prototype(),
                ErrorKind::Reference,
            ),
            JsNativeErrorKind::Syntax => {
                (constructors.syntax_error().prototype(), ErrorKind::Syntax)
            }
            JsNativeErrorKind::Type => (constructors.type_error().prototype(), ErrorKind::Type),
            JsNativeErrorKind::Uri => (constructors.uri_error().prototype(), ErrorKind::Uri),
            #[cfg(feature = "fuzz")]
            JsNativeErrorKind::NoInstructionsRemain => {
                unreachable!(
                    "The NoInstructionsRemain native error cannot be converted to an opaque type."
                )
            }
        };

        let o = JsObject::from_proto_and_data(prototype, ObjectData::error(tag));

        o.create_non_enumerable_data_property_or_throw("message", &**message, context);

        if let Some(cause) = cause {
            o.create_non_enumerable_data_property_or_throw(
                "cause",
                cause.to_opaque(context),
                context,
            );
        }

        if let JsNativeErrorKind::Aggregate(errors) = kind {
            let errors = errors
                .iter()
                .map(|e| e.to_opaque(context))
                .collect::<Vec<_>>();
            let errors = Array::create_array_from_list(errors, context);
            o.define_property_or_throw(
                "errors",
                PropertyDescriptor::builder()
                    .configurable(true)
                    .enumerable(false)
                    .writable(true)
                    .value(errors),
                context,
            )
            .expect("The spec guarantees this succeeds for a newly created object ");
        }
        o
    }
}

impl From<boa_parser::Error> for JsNativeError {
    fn from(err: boa_parser::Error) -> Self {
        Self::syntax().with_message(err.to_string())
    }
}

/// The list of possible error types a [`JsNativeError`] can be.
///
/// More information:
/// - [ECMAScript reference][spec]
/// - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-error-objects
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error
#[derive(Debug, Clone, Trace, Finalize)]
#[non_exhaustive]
pub enum JsNativeErrorKind {
    /// A collection of errors wrapped in a single error.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-aggregate-error-objects
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/AggregateError
    Aggregate(Vec<JsError>),
    /// A generic error. Commonly used as the base for custom exceptions.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-error-constructor
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error
    Error,
    /// An error related to the global function [`eval()`][eval].
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-evalerror
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/EvalError
    /// [eval]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/eval
    Eval,
    /// An error thrown when a value is outside its valid range.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-rangeerror
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RangeError
    Range,
    /// An error representing an invalid de-reference of a variable.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-referenceerror
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ReferenceError
    Reference,
    /// An error representing an invalid syntax in the Javascript language.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-syntaxerror
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SyntaxError
    Syntax,
    /// An error thrown when a variable or argument is not of a valid type.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-typeerror
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypeError
    Type,
    /// An error thrown when the [`encodeURI()`][e_uri] and [`decodeURI()`][d_uri] functions receive
    /// invalid parameters.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-urierror
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/URIError
    /// [e_uri]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/encodeURI
    /// [d_uri]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/decodeURI
    Uri,
    /// Error thrown when no instructions remain. Only used in a fuzzing context; not a valid JS
    /// error variant.
    #[cfg(feature = "fuzz")]
    NoInstructionsRemain,
}

impl std::fmt::Display for JsNativeErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsNativeErrorKind::Aggregate(_) => "AggregateError",
            JsNativeErrorKind::Error => "Error",
            JsNativeErrorKind::Eval => "EvalError",
            JsNativeErrorKind::Range => "RangeError",
            JsNativeErrorKind::Reference => "ReferenceError",
            JsNativeErrorKind::Syntax => "SyntaxError",
            JsNativeErrorKind::Type => "TypeError",
            JsNativeErrorKind::Uri => "UriError",
            #[cfg(feature = "fuzz")]
            JsNativeErrorKind::NoInstructionsRemain => "NoInstructionsRemain",
        }
        .fmt(f)
    }
}
