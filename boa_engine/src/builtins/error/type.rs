//! This module implements the global `TypeError` object.
//!
//! The `TypeError` object represents an error when an operation could not be performed,
//! typically (but not exclusively) when a value is not of the expected type.
//!
//! A `TypeError` may be thrown when:
//!  - an operand or argument passed to a function is incompatible with the type expected by that operator or function.
//!  - when attempting to modify a value that cannot be changed.
//!  - when attempting to use a value in an inappropriate way.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-typeerror
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypeError

use crate::{
    builtins::{
        function::{Function, FunctionKind},
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    error::JsNativeError,
    object::{internal_methods::get_prototype_from_constructor, JsObject, ObjectData, ObjectKind},
    property::Attribute,
    realm::Realm,
    string::utf16,
    Context, JsArgs, JsResult, JsString, JsValue, NativeFunction,
};
use boa_profiler::Profiler;

use super::{Error, ErrorKind};

/// JavaScript `TypeError` implementation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TypeError;

impl IntrinsicObject for TypeError {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        BuiltInBuilder::from_standard_constructor_static_shape::<Self>(
            realm,
            &boa_builtins::NATIVE_ERROR_CONSTRUCTOR_STATIC_SHAPE,
            &boa_builtins::NATIVE_ERROR_PROTOTYPE_STATIC_SHAPE,
        )
        .prototype(realm.intrinsics().constructors().error().constructor())
        .inherits(Some(realm.intrinsics().constructors().error().prototype()))
        .property(Self::NAME)
        .property(JsString::default())
        .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for TypeError {
    const NAME: &'static str = "TypeError";
}

impl BuiltInConstructor for TypeError {
    const LENGTH: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::type_error;

    /// Create a new error object.
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, let newTarget be the active function object; else let newTarget be NewTarget.
        let new_target = &if new_target.is_undefined() {
            context
                .vm
                .active_function
                .clone()
                .unwrap_or_else(|| {
                    context
                        .intrinsics()
                        .constructors()
                        .type_error()
                        .constructor()
                })
                .into()
        } else {
            new_target.clone()
        };
        // 2. Let O be ? OrdinaryCreateFromConstructor(newTarget, "%NativeError.prototype%", « [[ErrorData]] »).
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::type_error, context)?;
        let o = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            ObjectData::error(ErrorKind::Type),
        );

        // 3. If message is not undefined, then
        let message = args.get_or_undefined(0);
        if !message.is_undefined() {
            // a. Let msg be ? ToString(message).
            let msg = message.to_string(context)?;

            // b. Perform CreateNonEnumerableDataPropertyOrThrow(O, "message", msg).
            o.create_non_enumerable_data_property_or_throw(utf16!("message"), msg, context);
        }

        // 4. Perform ? InstallErrorCause(O, options).
        Error::install_error_cause(&o, args.get_or_undefined(1), context)?;

        // 5. Return O.
        Ok(o.into())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ThrowTypeError;

impl IntrinsicObject for ThrowTypeError {
    fn init(realm: &Realm) {
        fn throw_type_error(_: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
            Err(JsNativeError::typ()
                .with_message(
                    "'caller', 'callee', and 'arguments' properties may not be accessed on strict mode \
                    functions or the arguments objects for calls to them",
                )
                .into())
        }

        let obj = BuiltInBuilder::with_intrinsic::<Self>(realm)
            .prototype(realm.intrinsics().constructors().function().prototype())
            .static_property(utf16!("length"), 0, Attribute::empty())
            .static_property(utf16!("name"), "", Attribute::empty())
            .build();

        let mut obj = obj.borrow_mut();

        obj.extensible = false;
        *obj.kind_mut() = ObjectKind::Function(Function::new(
            FunctionKind::Native {
                function: NativeFunction::from_fn_ptr(throw_type_error),
                constructor: None,
            },
            realm.clone(),
        ));
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().throw_type_error().into()
    }
}
