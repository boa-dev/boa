//! This module implements the global `Symbol` object.
//!
//! The data type symbol is a primitive data type.
//! The `Symbol()` function returns a value of type symbol, has static properties that expose
//! several members of built-in objects, has static methods that expose the global symbol registry,
//! and resembles a built-in object class, but is incomplete as a constructor because it does not
//! support the syntax "`new Symbol()`".
//!
//! Every symbol value returned from `Symbol()` is unique.
//!
//! More information:
//! - [MDN documentation][mdn]
//! - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-symbol-value
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol

#[cfg(test)]
mod tests;

use crate::{
    builtins::{
        object::{
            internal_methods_trait::ObjectInternalMethods, Object, ObjectKind, INSTANCE_PROTOTYPE,
            PROTOTYPE,
        },
        value::{to_value, undefined, ResultValue, Value, ValueData},
    },
    exec::Interpreter,
};
use gc::{Gc, GcCell};
use rand::random;

/// Creates Symbol instances.
///
/// Symbol instances are ordinary objects that inherit properties from the Symbol prototype object.
/// Symbol instances have a `[[SymbolData]]` internal slot.
/// The `[[SymbolData]]` internal slot is the Symbol value represented by this Symbol object.
///
/// More information:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-symbol-description
pub fn call_symbol(_: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    // From an implementation and specificaition perspective Symbols are similar to Objects.
    // They have internal slots to hold the SymbolData and Description, they also have methods and a prototype.
    // So we start by creating an Object
    // TODO: Set prototype to Symbol.prototype (by changing to Object::create(), use interpreter to get Symbol.prototype)
    let mut sym_instance = Object::default();
    sym_instance.kind = ObjectKind::Symbol;

    // Set description which should either be undefined or a string
    let desc_string = match args.get(0) {
        Some(value) => to_value(value.to_string()),
        None => undefined(),
    };

    sym_instance.set_internal_slot("Description", desc_string);
    sym_instance.set_internal_slot("SymbolData", to_value(random::<i32>()));

    // Set __proto__ internal slot
    let proto = ctx
        .realm
        .global_obj
        .get_field_slice("Symbol")
        .get_field_slice(PROTOTYPE);
    sym_instance.set_internal_slot(INSTANCE_PROTOTYPE, proto);

    Ok(Gc::new(ValueData::Symbol(Box::new(GcCell::new(
        sym_instance,
    )))))
}

/// `Symbol.prototype.toString()`
///
/// This method returns a string representing the specified `Symbol` object.
///
/// /// More information:
/// - [MDN documentation][mdn]
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-symbol.prototype.tostring
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/toString
pub fn to_string(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    let s: Value = this.get_internal_slot("Description");
    let full_string = format!(r#"Symbol({})"#, s.to_string());
    Ok(to_value(full_string))
}

/// Create a new `Symbol` object.
pub fn create(global: &Value) -> Value {
    // Create prototype object
    let prototype = ValueData::new_obj(Some(global));
    make_builtin_fn!(to_string, named "toString", of prototype);
    make_constructor_fn!(call_symbol, call_symbol, global, prototype)
}

/// Initialise the `Symbol` object on the global object.
#[inline]
pub fn init(global: &Value) {
    global.set_field_slice("Symbol", create(global));
}
