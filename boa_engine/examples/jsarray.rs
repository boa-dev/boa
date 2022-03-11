use boa_engine::{
    object::{FunctionBuilder, JsArray},
    Context, JsValue,
};

fn main() -> Result<(), JsValue> {
    // We create a new `Context` to create a new Javascript executor.
    let context = &mut Context::default();

    // Create an empty array.
    let array = JsArray::new(context);

    assert!(array.is_empty(context)?);

    array.push("Hello, world", context)?; // [ "Hello, world" ]
    array.push(true, context)?; // [ "Hello, world", true ]

    assert!(!array.is_empty(context)?);

    assert_eq!(array.pop(context)?, JsValue::new(true)); // [ "Hello, world" ]
    assert_eq!(array.pop(context)?, JsValue::new("Hello, world")); // [ ]
    assert_eq!(array.pop(context)?, JsValue::undefined()); // [ ]

    array.push(1, context)?; // [ 1 ]

    assert_eq!(array.pop(context)?, JsValue::new(1)); // [ ]
    assert_eq!(array.pop(context)?, JsValue::undefined()); // [ ]

    array.push_items(
        &[
            JsValue::new(10),
            JsValue::new(11),
            JsValue::new(12),
            JsValue::new(13),
            JsValue::new(14),
        ],
        context,
    )?; // [ 10, 11, 12, 13, 14 ]

    array.reverse(context)?; // [ 14, 13, 12, 11, 10 ]

    assert_eq!(array.index_of(12, None, context)?, Some(2));

    // We can also use JsObject method `.get()` through the Deref trait.
    let element = array.get(2, context)?; // array[ 0 ]
    assert_eq!(element, JsValue::new(12));
    // Or we can use the `.at(index)` method.
    assert_eq!(array.at(0, context)?, JsValue::new(14)); // first element
    assert_eq!(array.at(-1, context)?, JsValue::new(10)); // last element

    // Join the array with an optional separator (default ",").
    let joined_array = array.join(None, context)?;
    assert_eq!(joined_array, "14,13,12,11,10");

    array.fill(false, Some(1), Some(4), context)?;

    let joined_array = array.join(Some("::".into()), context)?;
    assert_eq!(joined_array, "14::false::false::false::10");

    let filter_callback = FunctionBuilder::native(context, |_this, args, _context| {
        Ok(args.get(0).cloned().unwrap_or_default().is_number().into())
    })
    .build();

    let map_callback = FunctionBuilder::native(context, |_this, args, context| {
        args.get(0)
            .cloned()
            .unwrap_or_default()
            .pow(&JsValue::new(2), context)
    })
    .build();

    let mut data = Vec::new();
    for i in 1..=5 {
        data.push(JsValue::new(i));
    }
    let another_array = JsArray::from_iter(data, context); // [ 1, 2, 3, 4, 5]

    let chained_array = array // [ 14, false, false, false, 10 ]
        .filter(filter_callback, None, context)? // [ 14, 10 ]
        .map(map_callback, None, context)? // [ 196, 100 ]
        .sort(None, context)? // [ 100, 196 ]
        .concat(&[another_array.into()], context)? // [ 100, 196, 1, 2, 3, 4, 5 ]
        .slice(Some(1), Some(5), context)?; // [ 196, 1, 2, 3 ]

    assert_eq!(chained_array.join(None, context)?, "196,1,2,3");

    let reduce_callback = FunctionBuilder::native(context, |_this, args, context| {
        let accumulator = args.get(0).cloned().unwrap_or_default();
        let value = args.get(1).cloned().unwrap_or_default();

        accumulator.add(&value, context)
    })
    .build();

    assert_eq!(
        chained_array.reduce(reduce_callback, Some(JsValue::new(0)), context)?,
        JsValue::new(202)
    );

    context
        .global_object()
        .clone()
        .set("myArray", array, true, context)?;

    Ok(())
}
