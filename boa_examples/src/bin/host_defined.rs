// This example goes into the details on how to store user defined structs/state that is shared.

use boa_engine::{
    js_string, native_function::NativeFunction, Context, JsArgs, JsError, JsNativeError, Source,
};
use boa_gc::{Finalize, Trace};

/// Custom host-defined struct that has some state, and can be shared between JavaScript and rust.
#[derive(Default, Trace, Finalize)]
struct CustomHostDefinedStruct {
    #[unsafe_ignore_trace]
    counter: usize,
}

/// Custom host-defined struct that has some state, and can be shared between JavaScript and rust.
#[derive(Trace, Finalize)]
struct AnotherCustomHostDefinedStruct {
    #[unsafe_ignore_trace]
    counter: usize,
}

impl AnotherCustomHostDefinedStruct {
    fn new(value: usize) -> Self {
        Self { counter: value }
    }
}

fn main() -> Result<(), JsError> {
    // We create a new `Context` to create a new Javascript executor..
    let mut context = Context::default();

    // Get the realm from the context.
    let realm = context.realm().clone();

    // Insert a default CustomHostDefinedStruct.
    realm
        .host_defined()
        .insert_default::<CustomHostDefinedStruct>();

    {
        assert!(realm.host_defined().has::<CustomHostDefinedStruct>());

        // Get the [[HostDefined]] field from the realm and downcast it to our concrete type.
        let Some(host_defined) = realm.host_defined().get::<CustomHostDefinedStruct>() else {
            return Err(JsNativeError::typ()
                .with_message("Realm does not have HostDefined field")
                .into());
        };

        // Assert that the [[HostDefined]] field is in it's initial state.
        assert_eq!(host_defined.counter, 0);
    }

    // Insert another struct with state into [[HostDefined]] field.
    realm
        .host_defined()
        .insert(AnotherCustomHostDefinedStruct::new(10));

    {
        assert!(realm.host_defined().has::<AnotherCustomHostDefinedStruct>());

        // Get the [[HostDefined]] field from the realm and downcast it to our concrete type.
        let Some(host_defined) = realm.host_defined().get::<AnotherCustomHostDefinedStruct>()
        else {
            return Err(JsNativeError::typ()
                .with_message("Realm does not have HostDefined field")
                .into());
        };

        // Assert that the [[HostDefined]] field is in it's initial state.
        assert_eq!(host_defined.counter, 10);
    }

    // Remove a type from the [[HostDefined]] field.
    assert!(realm
        .host_defined()
        .remove::<AnotherCustomHostDefinedStruct>()
        .is_some());

    // Create and register function for getting and setting the realm value.
    //
    // The funtion lives in the context's realm and has access to the host-defined field.
    context.register_global_builtin_callable(
        js_string!("setRealmValue"),
        1,
        NativeFunction::from_fn_ptr(|_, args, context| {
            let value: usize = args.get_or_undefined(0).try_js_into(context)?;

            let host_defined = context.realm().host_defined();
            let Some(mut host_defined) = host_defined.get_mut::<CustomHostDefinedStruct>() else {
                return Err(JsNativeError::typ()
                    .with_message("Realm does not have HostDefined field")
                    .into());
            };

            host_defined.counter = value;

            Ok(value.into())
        }),
    )?;

    context.register_global_builtin_callable(
        js_string!("getRealmValue"),
        0,
        NativeFunction::from_fn_ptr(|_, _, context| {
            let host_defined = context.realm().host_defined();
            let Some(host_defined) = host_defined.get::<CustomHostDefinedStruct>() else {
                return Err(JsNativeError::typ()
                    .with_message("Realm does not have HostDefined field")
                    .into());
            };

            Ok(host_defined.counter.into())
        }),
    )?;

    // Run code in JavaScript that mutates the host-defined field on the Realm.
    context.eval(Source::from_bytes(
        r"
        setRealmValue(50);
        setRealmValue(getRealmValue() * 2);
    ",
    ))?;

    let Some(host_defined) = realm.host_defined().get::<CustomHostDefinedStruct>() else {
        return Err(JsNativeError::typ()
            .with_message("Realm does not have HostDefined field")
            .into());
    };

    // Assert that the host-defined field changed.
    assert_eq!(host_defined.counter, 100);

    Ok(())
}
