use boa_gc::{Finalize, Trace};

use crate::{
    builtins::{error::ErrorKind, Array},
    object::JsObject,
    object::ObjectData,
    property::PropertyDescriptor,
    syntax::parser,
    Context, JsResult, JsString, JsValue,
};

#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsError {
    inner: Repr,
}

#[derive(Debug, Clone, Trace, Finalize)]
enum Repr {
    Native(JsNativeError),
    Opaque(JsValue),
}

impl JsError {
    pub fn to_value(&self, context: &mut Context) -> JsValue {
        match &self.inner {
            Repr::Native(e) => e.to_value(context),
            Repr::Opaque(v) => v.clone(),
        }
    }

    // TODO: Should probably change this to return a custom error instead
    pub fn try_native(&self, context: &mut Context) -> JsResult<JsNativeError> {
        match &self.inner {
            Repr::Native(e) => Ok(e.clone()),
            Repr::Opaque(val) => {
                if let Some(obj) = val.as_object() {
                    if let Some(error) = obj.borrow().as_error() {
                        let message = if obj.has_property("message", context)? {
                            obj.get("message", context)?
                                .as_string()
                                .map(JsString::to_std_string)
                                .transpose()
                                .map_err(|_| {
                                    JsNativeError::typ().with_message(
                                        "field `message` cannot have unpaired surrogates",
                                    )
                                })?
                                .ok_or_else(|| {
                                    JsNativeError::typ()
                                        .with_message("invalid type for field `message`")
                                })?
                                .into()
                        } else {
                            "".into()
                        };

                        let cause = if obj.has_property("cause", context)? {
                            Some(obj.get("cause", context)?)
                        } else {
                            None
                        };

                        let kind = match error {
                            ErrorKind::Error => JsNativeErrorKind::Error,
                            ErrorKind::Eval => JsNativeErrorKind::Eval,
                            ErrorKind::Type => JsNativeErrorKind::Type,
                            ErrorKind::Range => JsNativeErrorKind::Range,
                            ErrorKind::Reference => JsNativeErrorKind::Reference,
                            ErrorKind::Syntax => JsNativeErrorKind::Syntax,
                            ErrorKind::Uri => JsNativeErrorKind::Uri,
                            ErrorKind::Aggregate => {
                                let errors = obj.get("errors", context)?;
                                let mut error_list = Vec::new();
                                match errors.as_object() {
                                    Some(errors) if errors.is_array() => {
                                        let length = errors.length_of_array_like(context)?;
                                        for i in 0..length {
                                            error_list.push(errors.get(i, context)?.into());
                                        }
                                    }
                                    _ => {
                                        return Err(JsNativeError::typ()
                                            .with_message(
                                                "field `errors` must be a valid Array object",
                                            )
                                            .into())
                                    }
                                }

                                JsNativeErrorKind::Aggregate(error_list)
                            }
                        };

                        return Ok(JsNativeError {
                            kind,
                            message,
                            cause,
                        });
                    }
                }
                Err(JsNativeError::typ()
                    .with_message("failed to convert value to native error")
                    .with_cause(val)
                    .into())
            }
        }
    }

    pub fn as_opaque(&self) -> Option<&JsValue> {
        match self.inner {
            Repr::Native(_) => None,
            Repr::Opaque(ref v) => Some(v),
        }
    }

    pub fn as_native(&self) -> Option<&JsNativeError> {
        match self.inner {
            Repr::Native(ref e) => Some(e),
            Repr::Opaque(_) => None,
        }
    }
}

impl From<parser::ParseError> for JsError {
    fn from(err: parser::ParseError) -> Self {
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

impl From<JsValue> for JsError {
    fn from(value: JsValue) -> Self {
        Self {
            inner: Repr::Opaque(value),
        }
    }
}

impl From<JsObject> for JsError {
    fn from(object: JsObject) -> Self {
        Self {
            inner: Repr::Opaque(object.into()),
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

#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsNativeError {
    pub kind: JsNativeErrorKind,
    message: Box<str>,
    cause: Option<JsValue>,
}

impl JsNativeError {
    fn new(kind: JsNativeErrorKind, message: Box<str>, cause: Option<JsValue>) -> Self {
        Self {
            kind,
            message,
            cause,
        }
    }

    pub fn aggregate(errors: Vec<JsError>) -> Self {
        Self::new(JsNativeErrorKind::Aggregate(errors), Box::default(), None)
    }
    pub fn error() -> Self {
        Self::new(JsNativeErrorKind::Error, Box::default(), None)
    }
    pub fn eval() -> Self {
        Self::new(JsNativeErrorKind::Eval, Box::default(), None)
    }
    pub fn range() -> Self {
        Self::new(JsNativeErrorKind::Range, Box::default(), None)
    }
    pub fn reference() -> Self {
        Self::new(JsNativeErrorKind::Reference, Box::default(), None)
    }
    pub fn syntax() -> Self {
        Self::new(JsNativeErrorKind::Syntax, Box::default(), None)
    }
    pub fn typ() -> Self {
        Self::new(JsNativeErrorKind::Type, Box::default(), None)
    }
    pub fn uri() -> Self {
        Self::new(JsNativeErrorKind::Uri, Box::default(), None)
    }

    #[must_use]
    pub fn with_message<S>(mut self, message: S) -> Self
    where
        S: Into<Box<str>>,
    {
        self.message = message.into();
        self
    }

    #[must_use]
    pub fn with_cause<V>(mut self, cause: V) -> Self
    where
        V: Into<JsValue>,
    {
        self.cause = Some(cause.into());
        self
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn cause(&self) -> Option<&JsValue> {
        self.cause.as_ref()
    }

    pub fn to_value(&self, context: &mut Context) -> JsValue {
        let Self {
            kind,
            message,
            cause,
        } = self;
        let constructors = context.intrinsics().constructors();
        let prototype = match kind {
            JsNativeErrorKind::Aggregate(_) => constructors.aggregate_error().prototype(),
            JsNativeErrorKind::Error => constructors.error().prototype(),
            JsNativeErrorKind::Eval => constructors.eval_error().prototype(),
            JsNativeErrorKind::Range => constructors.range_error().prototype(),
            JsNativeErrorKind::Reference => constructors.reference_error().prototype(),
            JsNativeErrorKind::Syntax => constructors.syntax_error().prototype(),
            JsNativeErrorKind::Type => constructors.type_error().prototype(),
            JsNativeErrorKind::Uri => constructors.uri_error().prototype(),
        };

        let o = JsObject::from_proto_and_data(prototype, ObjectData::error(kind.as_error_kind()));

        o.create_non_enumerable_data_property_or_throw("message", &**message, context);

        if let Some(cause) = cause {
            o.create_non_enumerable_data_property_or_throw("cause", cause.clone(), context);
        }

        if let JsNativeErrorKind::Aggregate(errors) = kind {
            let errors = errors
                .iter()
                .map(|e| e.to_value(context))
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
        o.into()
    }
}

impl std::fmt::Display for JsNativeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { kind, message, .. } = self;
        write!(f, "{kind}: {message}")
    }
}

impl From<parser::ParseError> for JsNativeError {
    fn from(err: parser::ParseError) -> Self {
        Self::syntax().with_message(err.to_string())
    }
}

#[derive(Debug, Clone, Trace, Finalize)]
#[non_exhaustive]
pub enum JsNativeErrorKind {
    Aggregate(Vec<JsError>),
    Error,
    Eval,
    Range,
    Reference,
    Syntax,
    Type,
    Uri,
}

impl JsNativeErrorKind {
    fn as_error_kind(&self) -> ErrorKind {
        match self {
            JsNativeErrorKind::Aggregate(_) => ErrorKind::Aggregate,
            JsNativeErrorKind::Error => ErrorKind::Error,
            JsNativeErrorKind::Eval => ErrorKind::Eval,
            JsNativeErrorKind::Range => ErrorKind::Range,
            JsNativeErrorKind::Reference => ErrorKind::Reference,
            JsNativeErrorKind::Syntax => ErrorKind::Syntax,
            JsNativeErrorKind::Type => ErrorKind::Type,
            JsNativeErrorKind::Uri => ErrorKind::Uri,
        }
    }
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
        }
        .fmt(f)
    }
}
