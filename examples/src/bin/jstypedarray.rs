// This example shows how to manipulate a Javascript array using Rust code.

use boa_engine::{
    js_string,
    native_function::NativeFunction,
    object::{
        builtins::{JsArray, JsArrayBuffer, JsUint8Array},
        FunctionObjectBuilder,
    },
    property::Attribute,
    Context, JsNativeError, JsResult, JsValue,
};
use boa_gc::{Gc, GcRefCell};

fn main() -> JsResult<()> {
    // We create a new `Context` to create a new Javascript executor.
    let context = &mut Context::default();

    let data: Vec<u8> = (0..=255).collect();

    let array = JsUint8Array::from_iter(data, context)?;

    assert_eq!(array.get(0, context)?, JsValue::ZERO);

    let mut sum = 0;

    for i in 0..=255 {
        assert_eq!(array.at(i, context)?, JsValue::new(i));
        sum += i;
    }

    let callback = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(|_this, args, context| {
            let accumulator = args.first().cloned().unwrap_or_default();
            let value = args.get(1).cloned().unwrap_or_default();

            accumulator.add(&value, context)
        }),
    )
    .build();

    assert_eq!(
        array.reduce(callback, Some(JsValue::ZERO), context)?,
        JsValue::new(sum)
    );

    let greater_than_10_predicate = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(|_this, args, _context| {
            let element = args
                .first()
                .cloned()
                .unwrap_or_default()
                .as_number()
                .expect("error at number conversion");
            Ok(JsValue::from(element > 10.0))
        }),
    )
    .build();

    assert_eq!(
        array.find_index(greater_than_10_predicate, None, context),
        Ok(Some(11))
    );

    let lower_than_200_predicate = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(|_this, args, _context| {
            let element = args
                .first()
                .cloned()
                .unwrap_or_default()
                .as_number()
                .expect("error at number conversion");
            Ok(JsValue::from(element < 200.0))
        }),
    )
    .build();

    assert_eq!(
        array.find_last(lower_than_200_predicate.clone(), None, context),
        Ok(JsValue::from(199u8))
    );

    let data: Vec<u8> = vec![90, 120, 150, 180, 210, 240];
    let array = JsUint8Array::from_iter(data, context)?;

    assert_eq!(
        array.find_last_index(lower_than_200_predicate, None, context),
        Ok(Some(3))
    );

    // forEach
    let array = JsUint8Array::from_iter(vec![1, 2, 3, 4, 5], context)?;
    let num_to_modify = Gc::new(GcRefCell::new(0u8));

    let js_function = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_copy_closure_with_captures(
            |_, args, captures, inner_context| {
                let element = args
                    .first()
                    .cloned()
                    .unwrap_or_default()
                    .to_uint8(inner_context)
                    .expect("error at number conversion");

                *captures.borrow_mut() += element;
                Ok(JsValue::UNDEFINED)
            },
            Gc::clone(&num_to_modify),
        ),
    )
    .build();

    let _unused = array.for_each(js_function, None, context);

    let borrow = *num_to_modify.borrow();
    assert_eq!(borrow, 15u8);

    // includes
    assert_eq!(array.includes(JsValue::new(2), None, context), Ok(true));
    let empty_array = JsUint8Array::from_iter(vec![], context)?;
    assert_eq!(
        empty_array.includes(JsValue::new(2), None, context),
        Ok(false)
    );

    // set
    let array_buffer8 = JsArrayBuffer::new(8, context)?;
    let initialized8_array = JsUint8Array::from_array_buffer(array_buffer8, context)?;
    initialized8_array.set_values(
        JsArray::from_iter(vec![JsValue::new(1), JsValue::new(2)], context).into(),
        Some(3),
        context,
    )?;
    assert_eq!(initialized8_array.get(0, context)?, JsValue::ZERO);
    assert_eq!(initialized8_array.get(1, context)?, JsValue::ZERO);
    assert_eq!(initialized8_array.get(2, context)?, JsValue::ZERO);
    assert_eq!(initialized8_array.get(3, context)?, JsValue::new(1.0));
    assert_eq!(initialized8_array.get(4, context)?, JsValue::new(2.0));
    assert_eq!(initialized8_array.get(5, context)?, JsValue::ZERO);
    assert_eq!(initialized8_array.get(6, context)?, JsValue::ZERO);
    assert_eq!(initialized8_array.get(7, context)?, JsValue::ZERO);
    assert_eq!(initialized8_array.get(8, context)?, JsValue::Undefined);

    // subarray
    let array = JsUint8Array::from_iter(vec![1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8], context)?;
    let subarray2_6 = array.subarray(2, 6, context)?;
    assert_eq!(subarray2_6.length(context)?, 4);
    assert_eq!(subarray2_6.get(0, context)?, JsValue::new(3.0));
    assert_eq!(subarray2_6.get(1, context)?, JsValue::new(4.0));
    assert_eq!(subarray2_6.get(2, context)?, JsValue::new(5.0));
    assert_eq!(subarray2_6.get(3, context)?, JsValue::new(6.0));

    let subarray4_6 = array.subarray(-4, 6, context)?;
    assert_eq!(subarray4_6.length(context)?, 2);
    assert_eq!(subarray4_6.get(0, context)?, JsValue::new(5.0));
    assert_eq!(subarray4_6.get(1, context)?, JsValue::new(6.0));

    // buffer
    let array_buffer8 = JsArrayBuffer::new(8, context)?;
    let array = JsUint8Array::from_array_buffer(array_buffer8, context)?;

    assert_eq!(
        array
            .buffer(context)?
            .as_object()
            .unwrap()
            .get(js_string!("byteLength"), context)
            .unwrap(),
        JsValue::new(8)
    );

    // constructor
    assert_eq!(
        Err(JsNativeError::typ()
            .with_message("the TypedArray constructor should never be called directly")
            .into()),
        array.constructor(context)
    );

    // copyWithin
    let array = JsUint8Array::from_iter(vec![1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8], context)?;
    array.copy_within(3, 1, Some(3), context)?;
    assert_eq!(array.get(0, context)?, JsValue::new(1.0));
    assert_eq!(array.get(1, context)?, JsValue::new(2.0));
    assert_eq!(array.get(2, context)?, JsValue::new(3.0));
    assert_eq!(array.get(3, context)?, JsValue::new(2.0));
    assert_eq!(array.get(4, context)?, JsValue::new(3.0));
    assert_eq!(array.get(5, context)?, JsValue::new(6.0));
    assert_eq!(array.get(6, context)?, JsValue::new(7.0));
    assert_eq!(array.get(7, context)?, JsValue::new(8.0));

    // toLocaleString
    // let array = JsUint32Array::from_iter(vec![500, 8123, 12], context)?;
    // let locales: Option<JsValue> = Some(js_string!("de-DE").into());
    // let options = Some(context.eval(Source::from_bytes(
    //     r##"let options = { style: "currency", currency: "EUR" }; options;"##,
    // ))?);
    // assert_eq!(
    //     array.to_locale_string(locales, options, context)?,
    //     js_string!("500,00 €,8.123,00 €,12,00 €").into()
    // );

    // toStringTag
    let array = JsUint8Array::from_iter(vec![1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8], context)?;
    let tag = array.to_string_tag(context)?.to_string(context)?;
    assert_eq!(tag, js_string!("Uint8Array"));

    context
        .register_global_property(
            js_string!("myUint8Array"),
            array,
            Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .unwrap();

    Ok(())
}
