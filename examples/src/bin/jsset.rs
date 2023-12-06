// This example shows how to manipulate a Javascript Set using Rust code.
#![allow(clippy::bool_assert_comparison)]
use boa_engine::{js_string, object::builtins::JsSet, Context, JsError, JsValue};

fn main() -> Result<(), JsError> {
    // New `Context` for a new Javascript executor.
    let context = &mut Context::default();

    // Create an empty set.
    let set = JsSet::new(context);

    assert_eq!(set.size()?, 0);
    set.add(5, context)?;
    assert_eq!(set.size()?, 1);
    set.add(10, context)?;
    assert_eq!(set.size()?, 2);
    set.clear(context)?;
    assert_eq!(set.size()?, 0);

    set.add(js_string!("one"), context)?;
    set.add(js_string!("two"), context)?;
    set.add(js_string!("three"), context)?;

    assert!(set.has(js_string!("one"), context)?);
    assert_eq!(set.has(js_string!("One"), context)?, false);

    set.delete(js_string!("two"), context)?;

    assert_eq!(set.has(js_string!("two"), context)?, false);

    set.clear(context)?;

    assert_eq!(set.has(js_string!("one"), context)?, false);
    assert_eq!(set.has(js_string!("three"), context)?, false);
    assert_eq!(set.size()?, 0);

    // Add a slice into a set;
    set.add_items(
        &[JsValue::new(1), JsValue::new(2), JsValue::new(3)],
        context,
    )?;
    // Will return 1, as one slice was added.
    assert_eq!(set.size()?, 1);

    // Make a new set from a slice
    let slice_set = JsSet::from_iter([JsValue::new(1), JsValue::new(2), JsValue::new(3)], context);
    // Will return 3, as each element of slice was added into the set.
    assert_eq!(slice_set.size()?, 3);

    set.clear(context)?;

    Ok(())
}
