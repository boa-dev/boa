// This example goes into the details on how to store user defined structs/state that is shared.

use boa_engine::{
    native_function::NativeFunction, Context, JsArgs, JsData, JsError, JsNativeError, JsString,
    JsValue, Source,
};
use boa_gc::{Finalize, Trace};

/// Custom host-defined struct that has some state, and can be shared between JavaScript and rust.
#[derive(Default, Trace, Finalize, JsData)]
struct CustomHostDefinedStruct {
    #[unsafe_ignore_trace]
    counter: usize,
}

/// Custom host-defined struct that has some state, and can be shared between JavaScript and rust.
#[derive(Trace, Finalize, JsData)]
struct AnotherCustomHostDefinedStruct {
    #[unsafe_ignore_trace]
    counter: usize,
}

impl AnotherCustomHostDefinedStruct {
    fn new(value: usize) -> Self {
        Self { counter: value }
    }
}

/// Custom host-defined struct that tracks the number of calls to the `getRealmValue` and `setRealmValue` functions.
#[derive(Default, Trace, Finalize, JsData)]
struct HostDefinedMetrics {
    #[unsafe_ignore_trace]
    counter: usize,
}

fn main() -> Result<(), JsError> {
    // We create a new `Context` to create a new Javascript executor..
    let mut context = Context::default();

    // Get the realm from the context.
    let realm = context.realm().clone();

    // Insert a default CustomHostDefinedStruct.
    realm
        .host_defined_mut()
        .insert_default::<CustomHostDefinedStruct>();

    {
        assert!(realm.host_defined().has::<CustomHostDefinedStruct>());

        // Get the [[HostDefined]] field from the realm and downcast it to our concrete type.
        let host_defined = realm.host_defined();
        let Some(host_defined) = host_defined.get::<CustomHostDefinedStruct>() else {
            return Err(JsNativeError::typ()
                .with_message("Realm does not have HostDefined field")
                .into());
        };

        // Assert that the [[HostDefined]] field is in it's initial state.
        assert_eq!(host_defined.counter, 0);
    }

    // Insert another struct with state into [[HostDefined]] field.
    realm
        .host_defined_mut()
        .insert(AnotherCustomHostDefinedStruct::new(10));

    {
        assert!(realm.host_defined().has::<AnotherCustomHostDefinedStruct>());

        // Get the [[HostDefined]] field from the realm and downcast it to our concrete type.
        let host_defined = realm.host_defined();
        let Some(host_defined) = host_defined.get::<AnotherCustomHostDefinedStruct>() else {
            return Err(JsNativeError::typ()
                .with_message("Realm does not have HostDefined field")
                .into());
        };

        // Assert that the [[HostDefined]] field is in it's initial state.
        assert_eq!(host_defined.counter, 10);
    }

    // Remove a type from the [[HostDefined]] field.
    assert!(realm
        .host_defined_mut()
        .remove::<AnotherCustomHostDefinedStruct>()
        .is_some());

    // Create and register function for getting and setting the realm value.
    //
    // The funtion lives in the context's realm and has access to the host-defined field.
    context.register_global_builtin_callable(
        JsString::from("setRealmValue"),
        1,
        NativeFunction::from_fn_ptr(|_, args, context| {
            let value: usize = args.get_or_undefined(0).try_js_into(context)?;

            let mut host_defined = context.realm().host_defined_mut();
            let (Some(host_defined), Some(metrics)) =
                host_defined.get_many_mut::<(CustomHostDefinedStruct, HostDefinedMetrics), 2>()
            else {
                return Err(JsNativeError::typ()
                    .with_message("Realm does not have HostDefined fields")
                    .into());
            };

            host_defined.counter = value;
            metrics.counter += 1;

            Ok(value.into())
        }),
    )?;

    context.register_global_builtin_callable(
        JsString::from("getRealmValue"),
        0,
        NativeFunction::from_fn_ptr(|_, _, context| {
            let mut host_defined = context.realm().host_defined_mut();

            let value: JsValue = {
                let Some(host_defined) = host_defined.get::<CustomHostDefinedStruct>() else {
                    return Err(JsNativeError::typ()
                        .with_message("Realm does not have HostDefined field")
                        .into());
                };
                host_defined.counter.into()
            };

            let Some(metrics) = host_defined.get_mut::<HostDefinedMetrics>() else {
                return Err(JsNativeError::typ()
                    .with_message("Realm does not have HostDefined field")
                    .into());
            };

            metrics.counter += 1;

            Ok(value)
        }),
    )?;

    // Insert HostDefinedMetrics into the [[HostDefined]] field.
    realm
        .host_defined_mut()
        .insert_default::<HostDefinedMetrics>();

    // Run code in JavaScript that mutates the host-defined field on the Realm.
    context.eval(Source::from_bytes(
        r"
        setRealmValue(50);
        setRealmValue(getRealmValue() * 2);
    ",
    ))?;

    let host_defined = realm.host_defined();
    let Some(host_defined_value) = host_defined.get::<CustomHostDefinedStruct>() else {
        return Err(JsNativeError::typ()
            .with_message("Realm does not have HostDefined field")
            .into());
    };

    // Assert that the host-defined field changed.
    assert_eq!(host_defined_value.counter, 100);

    let Some(metrics) = host_defined.get::<HostDefinedMetrics>() else {
        return Err(JsNativeError::typ()
            .with_message("Realm does not have HostDefined field")
            .into());
    };

    // Assert that we called the getRealmValue and setRealmValue functions (3 times in total)
    assert_eq!(metrics.counter, 3);

    Ok(())
}
