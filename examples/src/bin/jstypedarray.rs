// This example shows how to manipulate a Javascript array using Rust code.

use boa_engine::{
    js_string,
    native_function::NativeFunction,
    object::{builtins::JsUint8Array, FunctionObjectBuilder},
    property::Attribute,
    Context, JsResult, JsValue,
};
use boa_gc::{Gc, GcRefCell};

fn main() -> JsResult<()> {
    // We create a new `Context` to create a new Javascript executor.
    let context = &mut Context::default();

    let data: Vec<u8> = (0..=255).collect();

    let array = JsUint8Array::from_iter(data, context)?;

    assert_eq!(array.get(0, context)?, JsValue::new(0));

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
        array.reduce(callback, Some(JsValue::new(0)), context)?,
        JsValue::new(sum)
    );

    let greter_than_10_predicate = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(|_this, args, _context| {
            let element = args
                .get(0)
                .cloned()
                .unwrap_or_default()
                .as_number()
                .expect("error at number conversion");
            Ok(JsValue::Boolean(element > 10.0))
        }),
    )
    .build();

    assert_eq!(
        array.find_index(greter_than_10_predicate, None, context),
        Ok(Some(11))
    );

    let lower_than_200_predicate = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(|_this, args, _context| {
            let element = args
                .get(0)
                .cloned()
                .unwrap_or_default()
                .as_number()
                .expect("error at number conversion");
            Ok(JsValue::Boolean(element < 200.0))
        }),
    )
    .build();

    assert_eq!(
        array.find_last(lower_than_200_predicate.clone(), None, context),
        Ok(JsValue::Integer(199))
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
                    .get(0)
                    .cloned()
                    .unwrap_or_default()
                    .to_uint8(inner_context)
                    .expect("error at number conversion");

                *captures.borrow_mut() += element;
                Ok(JsValue::Undefined)
            },
            Gc::clone(&num_to_modify),
        ),
    )
    .build();

    let _unused = array.for_each(js_function, None, context);

    let borrow = *num_to_modify.borrow();
    assert_eq!(borrow, 15u8);

    context
        .register_global_property(
            js_string!("myUint8Array"),
            array,
            Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .unwrap();

    Ok(())
}
