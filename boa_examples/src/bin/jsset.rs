// This example shows how to manipulate a Javascript set using Rust code.

use boa_engine::{object::JsSet, Context, JsValue};

fn main() -> Result<(), JsValue> {
    // New `Context` for a new Javascript executor.
    let context = &mut Context::default();

    // Create an empty set.
    let set = JsSet::new(context);

    assert_eq!(set.size(context)?, 0);
    set.add(5, context)?;
    assert_eq!(set.size(context)?, 1);
    set.add(10, context)?;
    assert_eq!(set.size(context)?, 2);
    set.clear(context)?;
    assert_eq!(set.size(context)?, 0);

    set.add("one", context)?;
    set.add("two", context)?;
    set.add("three", context)?;

    assert!(set.has("one", context)?);
    assert_eq!(set.has("One", context)?, false);

    set.delete("two", context)?;

    assert_eq!(set.has("two", context)?, false);

    set.clear(context)?;

    assert_eq!(set.has("one", context)?, false);
    assert_eq!(set.has("three", context)?, false);
    assert_eq!(set.size(context)?, 0);

    Ok(())
}
